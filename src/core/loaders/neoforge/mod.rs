//! `NeoForge` mod support (`META-INF/neoforge.mods.toml`).

pub mod dep;

use super::forge_modern::metadata;
use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::ModType;
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// Handles `NeoForge` mods detected through `META-INF/neoforge.mods.toml`.
pub struct NeoForgeHandler;

impl ModHandler for NeoForgeHandler {
    fn mod_type(&self) -> ModType {
        ModType::NeoForge
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["META-INF/neoforge.mods.toml"]
    }

    fn metadata_files(&self) -> &'static [&'static str] {
        &["META-INF/neoforge.mods.toml", "META-INF/mods.toml"]
    }

    fn extract_deps(&self, raw: &str) -> Result<Vec<DepEntry>> {
        dep::extract(raw)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::NeoForge(Box::new(metadata::parse(raw)?)))
    }
}

pub(crate) fn is_neoforge_mods_toml(raw: &str) -> bool {
    let Ok(meta) = metadata::parse(raw) else {
        return false;
    };
    meta.mod_loader
        .as_deref()
        .is_some_and(|l| l.eq_ignore_ascii_case("neoforge"))
        || meta.dependencies.values().any(|deps| {
            deps.iter()
                .any(|d| d.mod_id.eq_ignore_ascii_case("neoforge"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_neoforge_mod_loader() {
        let toml = r#"
modLoader = "neoforge"
loaderVersion = "[1,)"
[[mods]]
modId = "example"
"#;
        assert!(is_neoforge_mods_toml(toml));
    }

    #[test]
    fn detects_neoforge_dependency() {
        let toml = r#"
modLoader = "javafml"
loaderVersion = "[1,)"
[[mods]]
modId = "example"
[[dependencies.example]]
modId = "neoforge"
mandatory = true
versionRange = "[20.2,)"
"#;
        assert!(is_neoforge_mods_toml(toml));
    }

    #[test]
    fn ignores_forge_dependency() {
        let toml = r#"
modLoader = "javafml"
loaderVersion = "[1,)"
[[mods]]
modId = "example"
[[dependencies.example]]
modId = "forge"
mandatory = true
versionRange = "[47,)"
"#;
        assert!(!is_neoforge_mods_toml(toml));
    }

    #[test]
    fn ignores_malformed_toml() {
        assert!(!is_neoforge_mods_toml("not toml"));
    }
}
