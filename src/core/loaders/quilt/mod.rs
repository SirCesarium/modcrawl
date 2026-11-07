//! Quilt mod support (`quilt.mod.json`).

pub mod dep;
pub mod metadata;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::ModType;
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// Quilt mods (`quilt.mod.json`).
pub struct QuiltHandler;

impl ModHandler for QuiltHandler {
    fn mod_type(&self) -> ModType {
        ModType::Quilt
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["quilt.mod.json"]
    }

    fn extract_deps(&self, raw: &str) -> Result<Vec<DepEntry>> {
        dep::extract(raw)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Quilt(Box::new(metadata::parse(raw)?)))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn handler_extracts_deps() {
        let json = r#"{
            "schema_version": 1,
            "quilt_loader": {
                "id": "testmod",
                "version": "1.0.0",
                "depends": ["dep_a"]
            }
        }"#;
        let deps = QuiltHandler.extract_deps(json).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "dep_a");
    }
}
