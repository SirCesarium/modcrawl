pub mod dep;
pub mod metadata;

use zipcrawl::ZipManager;

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::ModType;
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

pub struct FabricHandler;

impl ModHandler for FabricHandler {
    fn mod_type(&self) -> ModType {
        ModType::Fabric
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["fabric.mod.json"]
    }

    fn extract_deps(&self, mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
        dep::extract(mng)
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Fabric(Box::new(metadata::parse(raw)?)))
    }
}
