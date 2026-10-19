use serde_json::from_str;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use super::metadata::ForgeLegacyModEntry;
use crate::error::Result;

pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let entries = from_str::<Vec<ForgeLegacyModEntry>>(raw)?;
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
#[allow(clippy::unwrap_used, clippy::default_trait_access)]
mod tests {
    use super::*;
    use crate::core::dep::types::DepKind;

    #[test]
    fn extract_dependencies() {
        let json = r#"[
            {"modid": "mod_a", "dependencies": ["lib_x"]},
            {"modid": "mod_b", "dependencies": ["lib_x", "lib_y"]}
        ]"#;
        let deps = extract(json).unwrap();
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
        let deps = extract(json).unwrap();
        assert_eq!(deps, []);
    }
}
