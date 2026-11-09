//! Dependency extraction from `bungee.yml`.

use serde_saphyr::from_str;

use super::metadata::BungeePluginMetadata;
use crate::core::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

/// Extracts dependencies from a `BungeeCord` `bungee.yml` document.
///
/// # Errors
///
/// Returns an error when the YAML is malformed or does not match the schema.
pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let meta = from_str::<BungeePluginMetadata>(raw)?;
    let mut deps = Vec::new();

    for name in meta.depends {
        deps.push(DepEntry::new(
            name,
            DepKind::Required,
            VersionRange::parse(None),
        ));
    }
    for name in meta.soft_depends {
        deps.push(DepEntry::new(
            name,
            DepKind::Optional,
            VersionRange::parse(None),
        ));
    }

    Ok(deps)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::default_trait_access)]
mod tests {
    use super::*;

    #[test]
    fn extract_bungee_dependencies() {
        let yaml = r"
name: TestPlugin
version: 1.0.0
main: me.example.TestPlugin
depends: [RequiredDep]
softDepends: [OptionalDep]
";
        let deps = extract(yaml).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(
            deps.iter()
                .any(|d| d.name == "RequiredDep" && d.kind == DepKind::Required)
        );
        assert!(
            deps.iter()
                .any(|d| d.name == "OptionalDep" && d.kind == DepKind::Optional)
        );
    }

    #[test]
    fn extract_empty() {
        let yaml = r"
name: TestPlugin
version: 1.0.0
main: me.example.TestPlugin
";
        let deps = extract(yaml).unwrap();
        assert_eq!(deps, []);
    }
}
