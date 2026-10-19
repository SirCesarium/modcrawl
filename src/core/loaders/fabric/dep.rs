use serde_json::{Value, from_str};

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use super::metadata::FabricModMetadata;
use crate::error::Result;

fn ver(value: Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s),
        Value::Array(arr) => arr.into_iter().find_map(|v| v.as_str().map(String::from)),
        _ => None,
    }
}

pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let meta = from_str::<FabricModMetadata>(raw)?;
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
    use super::*;
    use crate::core::dep::types::DepKind;

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
        let deps = extract(json).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter()
                .any(|d| d.name == "fabric-api" && d.kind == DepKind::Required)
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "minecraft" && d.kind == DepKind::Required)
        );
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
        let deps = extract(json).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter()
                .any(|d| d.name == "sodium" && d.kind == DepKind::Recommended)
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "iris" && d.kind == DepKind::Suggested)
        );
    }

    #[test]
    fn extract_empty_when_no_deps() {
        let json = r#"{"schemaVersion": 1, "id": "testmod", "version": "1.0.0"}"#;
        let deps = extract(json).unwrap();
        assert_eq!(deps, []);
    }
}
