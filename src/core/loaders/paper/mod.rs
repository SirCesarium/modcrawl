//! Paper plugin support (`paper-plugin.yml`).

pub mod dep;
pub mod metadata;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::{ModType, PluginType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// Handles Paper plugins detected through `paper-plugin.yml`.
pub struct PaperHandler;

impl ModHandler for PaperHandler {
    fn mod_type(&self) -> ModType {
        ModType::Plugin(PluginType::Paper)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["paper-plugin.yml"]
    }

    fn extract_deps(&self, raw: &str) -> Result<Vec<DepEntry>> {
        dep::extract(raw)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Paper(metadata::parse(raw)?))
    }
}
