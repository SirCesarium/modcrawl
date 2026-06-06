use zipcrawl::ZipManager;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

pub fn extract(mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
    let raw = mng.read_to_string("plugin.yml")?;
    let meta = serde_saphyr::from_str::<super::metadata::BukkitPluginMetadata>(&raw)?;
    let mut deps = Vec::new();

    for name in meta.depend {
        deps.push(DepEntry {
            name,
            kind: DepKind::Required,
            version_range: VersionRange::parse(None),
        });
    }
    for name in meta.softdepend {
        deps.push(DepEntry {
            name,
            kind: DepKind::Optional,
            version_range: VersionRange::parse(None),
        });
    }
    for name in meta.loadbefore {
        deps.push(DepEntry {
            name,
            kind: DepKind::LoadBefore,
            version_range: VersionRange::parse(None),
        });
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
    fn extract_depend_softdepend_loadbefore() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: com.example.TestPlugin
depend: [Vault, WorldEdit]
softdepend: [PlaceholderAPI]
loadbefore: [OtherPlugin]
"#;
        let bytes = make_zip_bytes(&[("plugin.yml", yaml)]);
        let mut mng = ZipManager::from_reader(&mut Cursor::new(bytes)).unwrap();
        let deps = extract(&mut mng).unwrap();
        assert_eq!(deps.len(), 4);
        assert!(deps
            .iter()
            .any(|d| d.name == "Vault" && d.kind == DepKind::Required));
        assert!(deps
            .iter()
            .any(|d| d.name == "WorldEdit" && d.kind == DepKind::Required));
        assert!(deps
            .iter()
            .any(|d| d.name == "PlaceholderAPI" && d.kind == DepKind::Optional));
        assert!(deps
            .iter()
            .any(|d| d.name == "OtherPlugin" && d.kind == DepKind::LoadBefore));
    }

    #[test]
    fn extract_empty_when_no_deps() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: com.example.TestPlugin
"#;
        let bytes = make_zip_bytes(&[("plugin.yml", yaml)]);
        let mut mng = ZipManager::from_reader(&mut Cursor::new(bytes)).unwrap();
        let deps = extract(&mut mng).unwrap();
        assert!(deps.is_empty());
    }
}
