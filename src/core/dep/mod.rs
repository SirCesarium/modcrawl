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

    let mut dependencies = dedup_deps(REGISTRY.extract_deps(&mod_type, mng)?);

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
                out[pos].kind = dep.kind;
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
        stem == name || stem.starts_with(&prefix)
    })
}
