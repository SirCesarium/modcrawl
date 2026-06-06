pub mod dep;
pub mod metadata;

use zipcrawl::ZipManager;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::{ForgeModFormat, ModType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

pub struct ForgeModernHandler;

impl ModHandler for ForgeModernHandler {
    fn mod_type(&self) -> ModType {
        ModType::Forge(ForgeModFormat::ModsToml)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["META-INF/mods.toml"]
    }

    fn extract_deps(&self, mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
        dep::extract(mng)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::ForgeModern(Box::new(metadata::parse(raw)?)))
    }
}
