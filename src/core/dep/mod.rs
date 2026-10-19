pub mod types;

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use zipcrawl::ZipManager;

pub use types::{DepEntry, DepReport, JarInJar};

use crate::core::REGISTRY;
use crate::error::Result;

/// Analyze dependencies of a mod/plugin JAR.
///
/// When `include_jij` is `false`, jar-in-jar entries are still scanned
/// internally to filter bundled deps, but not included in the report.
///
/// # Errors
///
/// Returns an error if the file cannot be read or is not a valid ZIP archive.
pub fn analyze(path: &Path, include_jij: bool) -> Result<DepReport> {
    let mut mng = ZipManager::new(path)?;
    analyze_impl(&mut mng, include_jij)
}

/// Analyze dependencies from a ZIP archive read via a `Read` impl.
///
/// When `include_jij` is `false`, jar-in-jar entries are still scanned
/// internally to filter bundled deps, but not included in the report.
///
/// # Errors
///
/// Returns an error if the data cannot be read or is not a valid ZIP archive.
#[allow(dead_code)]
pub fn analyze_reader<R: Read>(reader: &mut R, include_jij: bool) -> Result<DepReport> {
    let mut mng = ZipManager::from_reader(reader)?;
    analyze_impl(&mut mng, include_jij)
}

fn analyze_impl(mng: &mut ZipManager, include_jij: bool) -> Result<DepReport> {
    let entries = mng.entries()?;
    let mod_type = REGISTRY.detect(&entries);

    let mut dependencies = match REGISTRY.metadata_file_path(&mod_type) {
        Some(file) => {
            let raw = mng.read_to_string(file)?;
            dedup_deps(REGISTRY.extract_deps(&mod_type, &raw)?)
        }
        None => Vec::new(),
    };

    let jar_in_jar = find_embedded_jars(&entries);
    if !jar_in_jar.is_empty() {
        dependencies.retain(|dep| !is_bundled(&dep.name, &jar_in_jar));
    }

    Ok(DepReport {
        dependencies,
        jar_in_jar: if include_jij { jar_in_jar } else { Vec::new() },
    })
}

/// Merge duplicate `DepEntry` values by name.
///
/// Higher-priority `DepKind` wins on conflict.
fn dedup_deps(raw: Vec<DepEntry>) -> Vec<DepEntry> {
    let mut out: Vec<DepEntry> = Vec::new();
    let mut indices: HashMap<String, usize> = HashMap::new();

    for dep in raw {
        if let Some(&pos) = indices.get(&dep.name) {
            if dep.kind.priority() > out[pos].kind.priority() {
                // The winning entry brings its own version range along; keeping
                // the loser's range would report the wrong constraint.
                out[pos].kind = dep.kind;
                out[pos].version_range = dep.version_range;
            }
        } else {
            indices.insert(dep.name.clone(), out.len());
            out.push(dep);
        }
    }

    out
}

fn find_embedded_jars(entries: &[zipcrawl::ZipEntry]) -> Vec<JarInJar> {
    entries
        .iter()
        .filter(|e| {
            !e.is_dir
                && Path::new(&e.name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
        })
        .map(|e| JarInJar {
            path: e.name.clone(),
        })
        .collect()
}

fn is_bundled(name: &str, jars: &[JarInJar]) -> bool {
    let prefix = format!("{name}-");

    jars.iter().any(|j| {
        let stem = Path::new(&j.path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        if stem == name {
            return true;
        }

        // A versioned bundle is `<name>-<version>.jar`, where the version starts
        // with a digit. Anything else after the dash (`-common`, `-core`) is a
        // different artifact, not a bundled copy of this dependency.
        stem.strip_prefix(&prefix)
            .is_some_and(|suffix| suffix.starts_with(|c: char| c.is_ascii_digit()))
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::dep::types::{DepKind, VersionRange};

    fn entry(name: &str, kind: DepKind) -> DepEntry {
        DepEntry::new(name, kind, VersionRange::parse(None))
    }

    fn entry_with_range(name: &str, kind: DepKind, range: &str) -> DepEntry {
        DepEntry::new(name, kind, VersionRange::parse(Some(range.to_owned())))
    }

    fn jar(path: &str) -> JarInJar {
        JarInJar {
            path: path.to_owned(),
        }
    }

    // ── dedup_deps ─────────────────────────────────────────────────────────

    #[test]
    fn dedup_empty() {
        assert_eq!(dedup_deps(vec![]), []);
    }

    #[test]
    fn dedup_no_duplicates() {
        let deps = vec![entry("a", DepKind::Required), entry("b", DepKind::Optional)];
        let result = dedup_deps(deps);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedup_higher_priority_wins() {
        let deps = vec![entry("x", DepKind::Optional), entry("x", DepKind::Required)];
        let result = dedup_deps(deps);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DepKind::Required);
    }

    #[test]
    fn dedup_same_kind_preserves_first() {
        let deps = vec![entry("x", DepKind::Optional), entry("x", DepKind::Optional)];
        let result = dedup_deps(deps);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DepKind::Optional);
    }

    #[test]
    fn dedup_mixed_unique_and_dupes() {
        let deps = vec![
            entry("a", DepKind::Required),
            entry("b", DepKind::Suggested),
            entry("a", DepKind::LoadBefore),
        ];
        let result = dedup_deps(deps);
        assert_eq!(result.len(), 2);
        // 'a' should be LoadBefore (priority 4 > Required's 5? No, Required is 5)
        // Wait: Required=5, LoadBefore=4. So Required should win.
        assert_eq!(result[0].kind, DepKind::Required);
        assert_eq!(result[0].name, "a");
        assert_eq!(result[1].name, "b");
    }

    #[test]
    fn dedup_keeps_the_winning_entries_version_range() {
        let deps = vec![
            entry_with_range("lib", DepKind::Suggested, ">=1.0"),
            entry_with_range("lib", DepKind::Required, ">=2.0"),
        ];

        let result = dedup_deps(deps);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DepKind::Required);
        assert_eq!(result[0].version_range.raw, Some(">=2.0".to_owned()));
    }

    #[test]
    fn dedup_result_is_the_same_regardless_of_input_order() {
        let suggested = entry_with_range("lib", DepKind::Suggested, ">=1.0");
        let required = entry_with_range("lib", DepKind::Required, ">=2.0");

        let low_first = dedup_deps(vec![suggested.clone(), required.clone()]);
        let high_first = dedup_deps(vec![required, suggested]);

        assert_eq!(low_first[0].kind, DepKind::Required);
        assert_eq!(low_first[0].version_range.raw, Some(">=2.0".to_owned()));
        assert_eq!(high_first[0].kind, DepKind::Required);
        assert_eq!(high_first[0].version_range.raw, Some(">=2.0".to_owned()));
    }

    #[test]
    fn dedup_equal_priority_keeps_first_version_range() {
        let deps = vec![
            entry_with_range("lib", DepKind::Optional, ">=1.0"),
            entry_with_range("lib", DepKind::Optional, ">=3.0"),
        ];

        let result = dedup_deps(deps);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DepKind::Optional);
        assert_eq!(result[0].version_range.raw, Some(">=1.0".to_owned()));
    }

    // ── find_embedded_jars ─────────────────────────────────────────────────

    fn zip_entry(name: &str, is_dir: bool) -> zipcrawl::ZipEntry {
        zipcrawl::ZipEntry {
            name: name.to_owned(),
            is_dir,
            size: 0,
            crc: 0,
        }
    }

    #[test]
    fn find_jars_empty_when_no_jars() {
        let entries = vec![zip_entry("META-INF/MANIFEST.MF", false)];
        assert_eq!(find_embedded_jars(&entries), []);
    }

    #[test]
    fn find_jars_finds_jar_files() {
        let entries = vec![
            zip_entry("META-INF/jars/foo.jar", false),
            zip_entry("META-INF/jars/bar.jar", false),
            zip_entry("fabric.mod.json", false),
        ];
        let jars = find_embedded_jars(&entries);
        assert_eq!(jars.len(), 2);
        assert_eq!(jars[0].path, "META-INF/jars/foo.jar");
        assert_eq!(jars[1].path, "META-INF/jars/bar.jar");
    }

    #[test]
    fn find_jars_case_insensitive() {
        let entries = vec![
            zip_entry("libs/foo.JAR", false),
            zip_entry("libs/bar.Jar", false),
        ];
        assert_eq!(find_embedded_jars(&entries).len(), 2);
    }

    #[test]
    fn find_jars_excludes_directories() {
        let entries = vec![zip_entry("libs/", true)];
        assert_eq!(find_embedded_jars(&entries), []);
    }

    #[test]
    fn find_jars_in_subdirectories() {
        let entries = vec![zip_entry("META-INF/jars/dep-1.0.jar", false)];
        let jars = find_embedded_jars(&entries);
        assert_eq!(jars.len(), 1);
        assert_eq!(jars[0].path, "META-INF/jars/dep-1.0.jar");
    }

    // ── is_bundled ─────────────────────────────────────────────────────────

    #[test]
    fn bundled_exact_stem_match() {
        assert!(is_bundled("foo", &[jar("foo.jar")]));
    }

    #[test]
    fn bundled_prefix_match() {
        assert!(is_bundled("foo", &[jar("foo-1.0.jar")]));
    }

    #[test]
    fn bundled_no_match() {
        assert!(!is_bundled("foo", &[jar("bar.jar")]));
    }

    #[test]
    fn bundled_empty_jars() {
        assert!(!is_bundled("foo", &[]));
    }

    #[test]
    fn bundled_match_in_subdir() {
        assert!(is_bundled(
            "flywheel",
            &[jar("META-INF/jars/flywheel-1.0.0.jar")]
        ));
    }

    #[test]
    fn a_shared_artifact_prefix_does_not_bundle_a_real_dependency() {
        assert!(!is_bundled("library", &[jar("library-common-2.0.jar")]));
    }

    #[test]
    fn a_multi_segment_version_stem_bundles_the_dependency() {
        assert!(is_bundled("fabric-api", &[jar("fabric-api-0.92.0.jar")]));
    }

    #[test]
    fn a_hyphenated_name_with_a_version_bundles_the_dependency() {
        assert!(is_bundled(
            "cloth-config",
            &[jar("cloth-config-16.0.0.jar")]
        ));
    }
}
