//! Forge mod support (`META-INF/mods.toml`).

pub mod dep;
pub mod metadata;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::{ForgeModFormat, ModType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// Handles Forge mods detected through `META-INF/mods.toml`.
pub struct ForgeHandler;

impl ModHandler for ForgeHandler {
    fn mod_type(&self) -> ModType {
        ModType::Forge(ForgeModFormat::ModsToml)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["META-INF/mods.toml"]
    }

    fn extract_deps(&self, raw: &str) -> Result<Vec<DepEntry>> {
        dep::extract(raw)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Forge(Box::new(metadata::parse(raw)?)))
    }
}
