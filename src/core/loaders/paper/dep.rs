//! Dependency extraction from `paper-plugin.yml`.

use serde_saphyr::from_str;

use super::metadata::PaperPluginMetadata;
use crate::core::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

/// Extracts dependencies from a Paper `paper-plugin.yml` document.
///
/// # Errors
///
/// Returns an error when the YAML is malformed or does not match the schema.
pub fn extract(raw: &str) -> Result<Vec<DepEntry>> {
    let meta = from_str::<PaperPluginMetadata>(raw)?;
    let mut deps = Vec::new();

    let Some(dependencies) = meta.dependencies else {
        return Ok(deps);
    };

    for (name, entry) in dependencies.bootstrap {
        let entry = entry.unwrap_or_default();
        let is_after = entry
            .load
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("after"));
        let is_required = entry.required.unwrap_or(true);
        let kind = if is_after {
            DepKind::LoadBefore
        } else if is_required {
            DepKind::Required
        } else {
            DepKind::Optional
        };
        deps.push(DepEntry::new(name, kind, VersionRange::parse(None)));
    }

    for (name, entry) in dependencies.server {
        let entry = entry.unwrap_or_default();
        let is_after = entry
            .load
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("after"));
        let is_required = entry.required.unwrap_or(true);
        let kind = if is_after {
            DepKind::LoadBefore
        } else if is_required {
            DepKind::Required
        } else {
            DepKind::Optional
        };
        deps.push(DepEntry::new(name, kind, VersionRange::parse(None)));
    }

    Ok(deps)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::default_trait_access)]
mod tests {
    use super::*;

    #[test]
    fn extract_paper_dependencies() {
        let yaml = r"
name: TestPlugin
version: '1.0.0'
main: com.example.TestPlugin
dependencies:
  bootstrap:
    BootDep:
      load: BEFORE
      required: true
  server:
    WorldEdit:
      load: BEFORE
      required: true
    PlaceholderAPI:
      required: false
    LatePlugin:
      load: AFTER
";
        let deps = extract(yaml).unwrap();
        assert_eq!(deps.len(), 4);

        assert!(
            deps.iter()
                .any(|d| d.name == "BootDep" && d.kind == DepKind::Required)
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
                .any(|d| d.name == "LatePlugin" && d.kind == DepKind::LoadBefore)
        );
    }

    #[test]
    fn extract_no_dependencies() {
        let yaml = r"
name: TestPlugin
version: '1.0.0'
main: com.example.TestPlugin
";
        let deps = extract(yaml).unwrap();
        assert_eq!(deps, []);
    }
}
