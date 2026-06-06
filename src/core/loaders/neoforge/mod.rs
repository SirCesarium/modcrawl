pub mod dep;

use zipcrawl::ZipManager;

use super::forge_modern::metadata;
use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::ModType;
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

pub struct NeoForgeHandler;

impl ModHandler for NeoForgeHandler {
    fn mod_type(&self) -> ModType {
        ModType::NeoForge
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["META-INF/neoforge.mods.toml"]
    }

    fn extract_deps(&self, mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
        dep::extract(mng)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::NeoForge(Box::new(metadata::parse(raw)?)))
    }
}
