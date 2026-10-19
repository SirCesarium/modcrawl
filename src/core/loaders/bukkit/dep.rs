use serde_saphyr::from_str;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use super::metadata::BukkitPluginMetadata;
use crate::error::Result;

pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let meta = from_str::<BukkitPluginMetadata>(raw)?;
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
#[allow(clippy::unwrap_used, clippy::default_trait_access)]
mod tests {
    use super::*;
    use crate::core::dep::types::DepKind;

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
        let deps = extract(yaml).unwrap();
        assert_eq!(deps.len(), 4);
        assert!(
            deps.iter()
                .any(|d| d.name == "Vault" && d.kind == DepKind::Required)
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "WorldEdit" && d.kind == DepKind::Required)
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "PlaceholderAPI" && d.kind == DepKind::Optional)
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "OtherPlugin" && d.kind == DepKind::LoadBefore)
        );
    }

    #[test]
    fn extract_empty_when_no_deps() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: com.example.TestPlugin
"#;
        let deps = extract(yaml).unwrap();
        assert_eq!(deps, []);
    }
}
