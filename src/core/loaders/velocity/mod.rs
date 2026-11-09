//! Velocity proxy plugin support (`velocity-plugin.json`).

pub mod dep;
pub mod metadata;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::{ModType, PluginType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// Handles Velocity proxy plugins detected through `velocity-plugin.json`.
pub struct VelocityHandler;

impl ModHandler for VelocityHandler {
    fn mod_type(&self) -> ModType {
        ModType::Plugin(PluginType::Velocity)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["velocity-plugin.json"]
    }

    fn extract_deps(&self, raw: &str) -> Result<Vec<DepEntry>> {
        dep::extract(raw)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Velocity(metadata::parse(raw)?))
    }
}
