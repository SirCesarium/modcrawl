//! `BungeeCord` proxy plugin support.

pub mod dep;
pub mod metadata;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::{ModType, PluginType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// Handles `BungeeCord` proxy plugins detected through `bungee.yml`.
pub struct BungeeHandler;

impl ModHandler for BungeeHandler {
    fn mod_type(&self) -> ModType {
        ModType::Plugin(PluginType::Bungee)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["bungee.yml"]
    }

    fn extract_deps(&self, raw: &str) -> Result<Vec<DepEntry>> {
        dep::extract(raw)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Bungee(metadata::parse(raw)?))
    }
}
