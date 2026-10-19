pub mod dep;
pub mod metadata;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::{ModType, PluginType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

pub struct BukkitHandler;

impl ModHandler for BukkitHandler {
    fn mod_type(&self) -> ModType {
        ModType::Plugin(PluginType::Bukkit)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["plugin.yml"]
    }

    fn extract_deps(&self, raw: &str) -> Result<Vec<DepEntry>> {
        dep::extract(raw)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Bukkit(metadata::parse(raw)?))
    }
}
