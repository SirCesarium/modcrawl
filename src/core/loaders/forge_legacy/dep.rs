use zipcrawl::ZipManager;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

pub fn extract(mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
    let raw = mng.read_to_string("mcmod.info")?;
    let entries = serde_json::from_str::<Vec<super::metadata::ForgeLegacyModEntry>>(&raw)?;
    let mut deps = Vec::new();

    for entry in entries {
        for dep_name in entry.dependencies {
            deps.push(DepEntry {
                name: dep_name,
                kind: DepKind::Required,
                version_range: VersionRange::parse(None),
            });
        }
    }

    Ok(deps)
}

#[cfg(test)]
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
    fn extract_dependencies() {
        let json = r#"[
            {"modid": "mod_a", "dependencies": ["lib_x"]},
            {"modid": "mod_b", "dependencies": ["lib_x", "lib_y"]}
        ]"#;
        let bytes = make_zip_bytes(&[("mcmod.info", json)]);
        let mut mng = ZipManager::from_reader(&mut Cursor::new(bytes)).unwrap();
        let deps = extract(&mut mng).unwrap();
        assert_eq!(deps.len(), 3);
        assert!(deps.iter().all(|d| d.kind == DepKind::Required));
        assert!(deps.iter().any(|d| d.name == "lib_x"));
        assert!(deps.iter().any(|d| d.name == "lib_y"));
    }

    #[test]
    fn extract_empty_when_no_deps() {
        let json = r#"[
            {"modid": "mod_a"},
            {"modid": "mod_b"}
        ]"#;
        let bytes = make_zip_bytes(&[("mcmod.info", json)]);
        let mut mng = ZipManager::from_reader(&mut Cursor::new(bytes)).unwrap();
        let deps = extract(&mut mng).unwrap();
        assert!(deps.is_empty());
    }
}
