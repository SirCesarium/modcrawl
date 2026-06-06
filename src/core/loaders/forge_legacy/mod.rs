pub mod dep;
pub mod metadata;

use zipcrawl::ZipManager;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::{ForgeModFormat, ModType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

pub struct ForgeLegacyHandler;

impl ModHandler for ForgeLegacyHandler {
    fn mod_type(&self) -> ModType {
        ModType::Forge(ForgeModFormat::McmodInfo)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["mcmod.info"]
    }

    fn extract_deps(&self, mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
        dep::extract(mng)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::ForgeLegacy(metadata::parse(raw)?))
    }
}
