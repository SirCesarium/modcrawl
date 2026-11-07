//! Dependency extraction from `quilt.mod.json`.

use serde_json::from_str;

use super::metadata::{QuiltModMetadata, QuiltVersions};
use crate::core::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let meta = from_str::<QuiltModMetadata>(raw)?;
    let mut deps = Vec::new();

    for dep in meta.quilt_loader.depends {
        let kind = if dep.optional() {
            DepKind::Optional
        } else {
            DepKind::Required
        };
        let version_range = VersionRange::parse(dep.versions().map(QuiltVersions::join));
        deps.push(DepEntry::new(dep.id(), kind, version_range));
    }

    Ok(deps)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn extract_depends() {
        let json = r#"{
            "schema_version": 1,
            "quilt_loader": {
                "id": "testmod",
                "version": "1.0.0",
                "depends": [
                    {"id": "quilt_loader", "versions": ">=0.19.1"},
                    {"id": "minecraft", "versions": [">=1.20", "<1.21"]},
                    {"id": "java", "versions": {"all": [">=17"]}},
                    "bare_dep"
                ]
            }
        }"#;
        let deps = extract(json).unwrap();
        assert_eq!(deps.len(), 4);
        assert_eq!(deps[0].name, "quilt_loader");
        assert_eq!(deps[0].kind, DepKind::Required);
        assert_eq!(deps[0].version_range.min.as_deref(), Some("0.19.1"));
        assert_eq!(deps[1].name, "minecraft");
        assert_eq!(deps[1].kind, DepKind::Required);
        assert_eq!(deps[1].version_range.raw.as_deref(), Some(">=1.20 | <1.21"));
        assert_eq!(deps[2].name, "java");
        assert_eq!(deps[2].kind, DepKind::Required);
        assert_eq!(deps[2].version_range.min.as_deref(), Some("17"));
        assert_eq!(deps[3].name, "bare_dep");
        assert_eq!(deps[3].kind, DepKind::Required);
        assert_eq!(deps[3].version_range.raw, None);
    }

    #[test]
    fn extract_optional() {
        let json = r#"{
            "schema_version": 1,
            "quilt_loader": {
                "id": "testmod",
                "version": "1.0.0",
                "depends": [
                    {"id": "opt_mod", "optional": true, "versions": ">=1.0.0"}
                ]
            }
        }"#;
        let deps = extract(json).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "opt_mod");
        assert_eq!(deps[0].kind, DepKind::Optional);
        assert_eq!(deps[0].version_range.min.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn extract_empty_when_no_deps() {
        let json = r#"{
            "schema_version": 1,
            "quilt_loader": {
                "id": "testmod",
                "version": "1.0.0"
            }
        }"#;
        let deps = extract(json).unwrap();
        assert_eq!(deps, []);
    }
}
