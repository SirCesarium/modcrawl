//! Dependency extraction from `mods.toml`.

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use super::metadata::parse;
use crate::error::Result;

/// Extracts dependencies from a Forge `mods.toml` document.
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

pub(crate) fn forge_dep_kind(
    dep_type: Option<&str>,
    mandatory: Option<bool>,
    ordering: Option<&str>,
) -> DepKind {
    let lower_type = dep_type.map(str::to_lowercase);
    match lower_type.as_deref() {
        Some("incompatible") => DepKind::Incompatible,
        Some("discouraged") => DepKind::Discouraged,
        _ => {
            if ordering.is_some_and(|o| o.eq_ignore_ascii_case("BEFORE")) {
                DepKind::LoadBefore
            } else if matches!(lower_type.as_deref(), Some("optional")) || mandatory == Some(false)
            {
                DepKind::Optional
            } else {
                DepKind::Required
            }
        }
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
    fn extract_incompatible() {
        let toml = r#"
[[mods]]
modId = "testmod"
version = "1.0.0"

[[dependencies.testmod]]
modId = "bad-mod"
type = "incompatible"
versionRange = "<1.0"
"#;
        let deps = extract(toml).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "bad-mod");
        assert_eq!(deps[0].kind, DepKind::Incompatible);
        assert_eq!(deps[0].version_range.raw.as_deref(), Some("<1.0"));
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
