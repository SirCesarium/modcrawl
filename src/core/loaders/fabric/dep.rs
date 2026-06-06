use zipcrawl::ZipManager;

use serde_json::Value;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

fn ver(value: Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s),
        Value::Array(arr) => arr.into_iter().find_map(|v| v.as_str().map(String::from)),
        _ => None,
    }
}

pub fn extract(mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
    let raw = mng.read_to_string("fabric.mod.json")?;
    let meta = serde_json::from_str::<super::metadata::FabricModMetadata>(&raw)?;
    let mut deps = Vec::new();

    for (name, range) in meta.depends {
        deps.push(DepEntry::new(
            name,
            DepKind::Required,
            VersionRange::parse(ver(range)),
        ));
    }
    for (name, range) in meta.recommends {
        deps.push(DepEntry::new(
            name,
            DepKind::Recommended,
            VersionRange::parse(ver(range)),
        ));
    }
    for (name, range) in meta.suggests {
        deps.push(DepEntry::new(
            name,
            DepKind::Suggested,
            VersionRange::parse(ver(range)),
        ));
    }

    Ok(deps)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::default_trait_access)]
mod tests {
    use std::io::{Cursor, Write};
    use zip::ZipWriter;

    use super::*;
    use crate::core::dep::types::DepKind;

    fn make_zip_bytes(contents: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut buf);
        for (name, content) in contents {
            zip.start_file::<&str, ()>(name, Default::default())
                .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        buf.into_inner()
    }

    #[test]
    fn extract_depends() {
        let json = r#"{
            "schemaVersion": 1,
            "id": "testmod",
            "version": "1.0.0",
            "depends": {
                "fabric-api": ">=0.50.0",
                "minecraft": "~1.20.1"
            }
        }"#;
        let bytes = make_zip_bytes(&[("fabric.mod.json", json)]);
        let mut mng = ZipManager::from_reader(&mut Cursor::new(bytes)).unwrap();
        let deps = extract(&mut mng).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps
            .iter()
            .any(|d| d.name == "fabric-api" && d.kind == DepKind::Required));
        assert!(deps
            .iter()
            .any(|d| d.name == "minecraft" && d.kind == DepKind::Required));
    }

    #[test]
    fn extract_recommends_and_suggests() {
        let json = r#"{
            "schemaVersion": 1,
            "id": "testmod",
            "version": "1.0.0",
            "recommends": {
                "sodium": "*"
            },
            "suggests": {
                "iris": "*"
            }
        }"#;
        let bytes = make_zip_bytes(&[("fabric.mod.json", json)]);
        let mut mng = ZipManager::from_reader(&mut Cursor::new(bytes)).unwrap();
        let deps = extract(&mut mng).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps
            .iter()
            .any(|d| d.name == "sodium" && d.kind == DepKind::Recommended));
        assert!(deps
            .iter()
            .any(|d| d.name == "iris" && d.kind == DepKind::Suggested));
    }

    #[test]
    fn extract_empty_when_no_deps() {
        let json = r#"{"schemaVersion": 1, "id": "testmod", "version": "1.0.0"}"#;
        let bytes = make_zip_bytes(&[("fabric.mod.json", json)]);
        let mut mng = ZipManager::from_reader(&mut Cursor::new(bytes)).unwrap();
        let deps = extract(&mut mng).unwrap();
        assert!(deps.is_empty());
    }
}
