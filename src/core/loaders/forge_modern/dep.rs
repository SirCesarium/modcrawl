use toml::from_str;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use super::metadata::ModsTomlMetadata;
use crate::error::Result;

pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let meta = from_str::<ModsTomlMetadata>(raw)?;
    let mut deps = Vec::new();

    for deps_vec in meta.dependencies.values() {
        for dep in deps_vec {
            let kind = forge_dep_kind(dep.dep_type.as_deref());
            if kind.is_excluded() {
                continue;
            }
            deps.push(DepEntry {
                name: dep.mod_id.clone(),
                kind,
                version_range: VersionRange::parse(dep.version_range.clone()),
            });
        }
    }

    Ok(deps)
}

pub(crate) fn forge_dep_kind(dep_type: Option<&str>) -> DepKind {
    match dep_type {
        Some("optional") => DepKind::Optional,
        Some("incompatible") => DepKind::Incompatible,
        Some("discouraged") => DepKind::Discouraged,
        _ => DepKind::Required,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::default_trait_access)]
mod tests {
    use super::*;
    use crate::core::dep::types::DepKind;

    #[test]
    fn extract_required_and_optional() {
        let toml = r#"
[[mods]]
modId = "testmod"
version = "1.0.0"

[[dependencies.testmod]]
modId = "minecraft"
type = "required"
versionRange = ">=1.20"
mandatory = true

[[dependencies.testmod]]
modId = "optional-lib"
type = "optional"
versionRange = ">=2.0"
"#;
        let deps = extract(toml).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter()
                .any(|d| d.name == "minecraft" && d.kind == DepKind::Required)
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "optional-lib" && d.kind == DepKind::Optional)
        );
    }

    #[test]
    fn extract_skips_incompatible() {
        let toml = r#"
[[mods]]
modId = "testmod"
version = "1.0.0"

[[dependencies.testmod]]
modId = "bad-mod"
type = "incompatible"
"#;
        let deps = extract(toml).unwrap();
        assert_eq!(deps, []);
    }

    #[test]
    fn extract_empty_when_no_deps() {
        let toml = r#"
[[mods]]
modId = "testmod"
version = "1.0.0"
"#;
        let deps = extract(toml).unwrap();
        assert_eq!(deps, []);
    }
}
