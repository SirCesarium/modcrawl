//! Dependency extraction from `neoforge.mods.toml`.

use super::super::super::dep::types::{DepEntry, VersionRange};
use super::super::forge_modern::dep::forge_dep_kind;
use super::super::forge_modern::metadata::parse;
use crate::error::Result;

/// Extracts dependencies from a `NeoForge` `mods.toml` document.
///
/// # Errors
///
/// Returns an error when the TOML is malformed or does not match the schema.
pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let meta = parse(raw)?;
    let mut deps = Vec::new();

    for deps_vec in meta.dependencies.values() {
        for dep in deps_vec {
            let kind = forge_dep_kind(
                dep.dep_type.as_deref(),
                dep.mandatory,
                dep.ordering.as_deref(),
            );
            deps.push(DepEntry {
                name: dep.mod_id.clone(),
                kind,
                version_range: VersionRange::parse(dep.version_range.clone()),
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
    fn extract_required_and_optional() {
        let toml = r#"
[[mods]]
modId = "testmod"
version = "1.0.0"

[[dependencies.testmod]]
modId = "minecraft"
type = "required"
versionRange = ">=1.21"
mandatory = true

[[dependencies.testmod]]
modId = "opt-lib"
type = "optional"
versionRange = ">=3.0"
"#;
        let deps = extract(toml).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter()
                .any(|d| d.name == "minecraft" && d.kind == DepKind::Required)
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "opt-lib" && d.kind == DepKind::Optional)
        );
    }

    #[test]
    fn extract_mandatory_false_without_type() {
        let toml = r#"
[[mods]]
modId = "testmod"

[[dependencies.testmod]]
modId = "soft-dep"
mandatory = false
"#;
        let deps = extract(toml).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "soft-dep");
        assert_eq!(deps[0].kind, DepKind::Optional);
    }

    #[test]
    fn extract_ordering_before() {
        let toml = r#"
[[mods]]
modId = "testmod"

[[dependencies.testmod]]
modId = "other-mod"
ordering = "BEFORE"
"#;
        let deps = extract(toml).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "other-mod");
        assert_eq!(deps[0].kind, DepKind::LoadBefore);
    }

    #[test]
    fn extract_discouraged() {
        let toml = r#"
[[mods]]
modId = "testmod"

[[dependencies.testmod]]
modId = "bad-mod"
type = "discouraged"
"#;
        let deps = extract(toml).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "bad-mod");
        assert_eq!(deps[0].kind, DepKind::Discouraged);
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
