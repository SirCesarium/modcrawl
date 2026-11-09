//! Dependency extraction from `velocity-plugin.json`.

use serde_json::from_str;

use super::metadata::VelocityPluginMetadata;
use crate::core::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

/// Extracts dependencies from a Velocity `velocity-plugin.json` document.
///
/// # Errors
///
/// Returns an error when the JSON is malformed or does not match the schema.
pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let meta = from_str::<VelocityPluginMetadata>(raw)?;
    let mut deps = Vec::new();

    for d in meta.dependencies {
        let kind = if d.optional {
            DepKind::Optional
        } else {
            DepKind::Required
        };
        deps.push(DepEntry::new(d.id, kind, VersionRange::parse(None)));
    }

    Ok(deps)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::default_trait_access)]
mod tests {
    use super::*;

    #[test]
    fn extract_velocity_dependencies() {
        let json = r#"{
            "id": "test_plugin",
            "dependencies": [
                { "id": "luckperms", "optional": false },
                { "id": "viaversion", "optional": true }
            ]
        }"#;
        let deps = extract(json).unwrap();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "luckperms");
        assert_eq!(deps[0].kind, DepKind::Required);
        assert_eq!(deps[1].name, "viaversion");
        assert_eq!(deps[1].kind, DepKind::Optional);
    }

    #[test]
    fn extract_empty() {
        let json = r#"{"id": "test_plugin"}"#;
        let deps = extract(json).unwrap();
        assert_eq!(deps, []);
    }
}
