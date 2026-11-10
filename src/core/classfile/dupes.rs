//! Duplicate-class detection.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use serde::Serialize;
use zipcrawl::ZipManager;

use super::is_class_entry;
use crate::error::Result;

/// A class entry that appears in more than one JAR.
#[derive(Debug, Clone, Serialize)]
pub struct DupEntry {
    /// Archive-relative path of the duplicated class.
    pub class_name: String,
    /// Labels of the JARs that contain the same class.
    pub files: Vec<String>,
}

/// A pair of JARs that share one or more classes, with the list of shared classes.
#[derive(Debug, Clone, Serialize)]
pub struct ConflictEntry {
    /// Label of the first JAR (lexicographically smaller).
    pub file_a: String,
    /// Label of the second JAR.
    pub file_b: String,
    /// Classes shared by both JARs.
    pub classes: Vec<String>,
}

/// Group duplicate entries by conflicting JAR pair.
///
/// Each `ConflictEntry` names two JARs (`file_a` ≤ `file_b` lexicographically)
/// and the classes they share.  Pairs where both labels are identical are
/// excluded.
#[must_use]
pub fn group_conflicts(entries: &[DupEntry]) -> Vec<ConflictEntry> {
    let mut map: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    for entry in entries {
        for i in 0..entry.files.len() {
            for j in (i + 1)..entry.files.len() {
                let (a, b) = if entry.files[i] <= entry.files[j] {
                    (entry.files[i].clone(), entry.files[j].clone())
                } else {
                    (entry.files[j].clone(), entry.files[i].clone())
                };

                if a == b {
                    continue;
                }

                map.entry((a, b))
                    .or_default()
                    .push(entry.class_name.clone());
            }
        }
    }

    map.into_iter()
        .map(|((file_a, file_b), classes)| ConflictEntry {
            file_a,
            file_b,
            classes,
        })
        .collect()
}

/// Collects class entries across archives, one archive at a time.
///
/// An aggregating command processes several inputs, so it needs to add them
/// individually and report which one failed instead of aborting the run.
#[derive(Default)]
pub(crate) struct ClassIndex {
    classes: BTreeMap<String, Vec<String>>,
}

impl ClassIndex {
    /// Add every `.class` entry in the JAR at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the JAR cannot be read.
    pub(crate) fn add_path(&mut self, path: &Path) -> Result<()> {
        let mut manager = ZipManager::new(path)?;

        let label = path.to_string_lossy().to_string();

        self.add_manager(&mut manager, &label)
    }

    /// Add every `.class` entry in an in-memory archive labeled by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is not a valid ZIP archive.
    pub(crate) fn add_buffer(&mut self, label: &str, bytes: &[u8]) -> Result<()> {
        let mut cursor = Cursor::new(bytes);
        let mut manager = ZipManager::from_reader(&mut cursor)?;

        self.add_manager(&mut manager, label)
    }

    fn add_manager<R: Read + Seek>(
        &mut self,
        manager: &mut ZipManager<R>,
        label: &str,
    ) -> Result<()> {
        let entries = manager.entries()?;

        for entry in &entries {
            if is_class_entry(&entry.name) && !entry.is_dir {
                let files = self.classes.entry(entry.name.clone()).or_default();
                if !files.iter().any(|existing| existing == label) {
                    files.push(label.to_string());
                }
            }
        }

        Ok(())
    }

    /// Classes that appear in more than one archive.
    pub(crate) fn into_duplicates(self) -> Vec<DupEntry> {
        self.classes
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .map(|(class_name, files)| DupEntry { class_name, files })
            .collect()
    }
}

/// Find duplicate class entries across multiple JARs on disk.
///
/// # Errors
///
/// Returns an error if any JAR cannot be read.
pub fn find_duplicates(paths: &[&Path]) -> Result<Vec<DupEntry>> {
    let mut index = ClassIndex::default();

    for &path in paths {
        index.add_path(path)?;
    }

    Ok(index.into_duplicates())
}

/// Find duplicate class entries across in-memory JARs labeled by name.
///
/// # Errors
///
/// Returns an error if any buffer is not a valid ZIP archive.
pub fn find_duplicates_buffers(jars: &[(&str, &[u8])]) -> Result<Vec<DupEntry>> {
    let mut index = ClassIndex::default();

    for &(label, bytes) in jars {
        index.add_buffer(label, bytes)?;
    }

    Ok(index.into_duplicates())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::env;
    use std::fs;
    use std::io::Write;
    use std::process;

    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;

    /// A valid class file header plus padding: `collect_classes` only inspects
    /// entry names, so the body is never parsed.
    const FAKE_CLASS: &[u8] = b"\xca\xfe\xba\xbe\x00\x00\x00\x34 padding";

    /// Build a JAR entirely in memory from `(entry name, contents)` pairs.
    fn jar(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

        for (name, contents) in entries {
            zip.start_file(*name, SimpleFileOptions::default()).unwrap();
            zip.write_all(contents).unwrap();
        }

        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn buffers_report_classes_shared_between_jars() {
        let a = jar(&[
            ("com/example/Shared.class", FAKE_CLASS),
            ("com/example/OnlyInA.class", FAKE_CLASS),
        ]);
        let b = jar(&[("com/example/Shared.class", FAKE_CLASS)]);

        let entries = find_duplicates_buffers(&[("a.jar", &a), ("b.jar", &b)]).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].class_name, "com/example/Shared.class");
        assert_eq!(entries[0].files, ["a.jar", "b.jar"]);
    }

    #[test]
    fn buffers_single_jar_yields_no_dupes() {
        let a = jar(&[("com/example/Only.class", FAKE_CLASS)]);

        assert!(
            find_duplicates_buffers(&[("a.jar", &a)])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn buffers_empty_input_yields_no_dupes() {
        assert!(find_duplicates_buffers(&[]).unwrap().is_empty());
    }

    #[test]
    fn paths_with_same_file_name_stay_distinct() {
        let pid = process::id();
        let root = env::temp_dir().join(format!("modcrawl_dupes_same_name_{pid}"));
        let client = root.join("client");
        let server = root.join("server");
        fs::create_dir_all(&client).unwrap();
        fs::create_dir_all(&server).unwrap();

        let bytes = jar(&[("com/example/Shared.class", FAKE_CLASS)]);
        let client_jar = client.join("foo.jar");
        let server_jar = server.join("foo.jar");
        fs::write(&client_jar, &bytes).unwrap();
        fs::write(&server_jar, &bytes).unwrap();

        let entries = find_duplicates(&[&client_jar, &server_jar]).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].files,
            [
                client_jar.to_string_lossy().to_string(),
                server_jar.to_string_lossy().to_string()
            ]
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn adding_the_same_archive_twice_does_not_duplicate_its_label() {
        let pid = process::id();
        let path = env::temp_dir().join(format!("modcrawl_dupes_same_archive_{pid}.jar"));
        fs::write(&path, jar(&[("com/example/Only.class", FAKE_CLASS)])).unwrap();

        let entries = find_duplicates(&[&path, &path]).unwrap();

        assert!(entries.is_empty());

        fs::remove_file(&path).unwrap();
    }
}
