use std::collections::BTreeMap;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use serde::Serialize;
use zipcrawl::ZipManager;

use super::is_class_entry;
use crate::error::Result;

#[derive(Debug, Clone, Serialize)]
pub struct DupEntry {
    pub class_name: String,
    pub files: Vec<String>,
}

/// Find duplicate class entries across multiple JARs on disk.
///
/// # Errors
///
/// Returns an error if any JAR cannot be read.
pub fn find_duplicates(paths: &[&Path]) -> Result<Vec<DupEntry>> {
    let mut acc = BTreeMap::new();

    for &path in paths {
        let mut manager = ZipManager::new(path)?;

        let label = path.file_name().map_or_else(
            || path.to_string_lossy().to_string(),
            |n| n.to_string_lossy().to_string(),
        );

        collect_classes(&mut manager, &label, &mut acc)?;
    }

    Ok(into_dupes(acc))
}

/// Find duplicate class entries across in-memory JARs labeled by name.
///
/// # Errors
///
/// Returns an error if any buffer is not a valid ZIP archive.
pub fn find_duplicates_buffers(jars: &[(&str, &[u8])]) -> Result<Vec<DupEntry>> {
    let mut acc = BTreeMap::new();

    for &(label, bytes) in jars {
        let mut cursor = Cursor::new(bytes);
        let mut manager = ZipManager::from_reader(&mut cursor)?;

        collect_classes(&mut manager, label, &mut acc)?;
    }

    Ok(into_dupes(acc))
}

fn collect_classes<R: Read + Seek>(
    manager: &mut ZipManager<R>,
    label: &str,
    acc: &mut BTreeMap<String, Vec<String>>,
) -> Result<()> {
    let entries = manager.entries()?;

    for entry in &entries {
        if is_class_entry(&entry.name) && !entry.is_dir {
            acc.entry(entry.name.clone())
                .or_default()
                .push(label.to_string());
        }
    }

    Ok(())
}

fn into_dupes(acc: BTreeMap<String, Vec<String>>) -> Vec<DupEntry> {
    acc.into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(class_name, files)| DupEntry { class_name, files })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::Write;

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
            zip.start_file(*name, SimpleFileOptions::default())
                .unwrap();
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

        assert!(find_duplicates_buffers(&[("a.jar", &a)]).unwrap().is_empty());
    }

    #[test]
    fn buffers_empty_input_yields_no_dupes() {
        assert!(find_duplicates_buffers(&[]).unwrap().is_empty());
    }
}
