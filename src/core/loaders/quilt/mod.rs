pub mod metadata;

use crate::core::detect_mod_type::ModType;
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// Quilt mods (`quilt.mod.json`). Metadata only; dependency extraction is left
/// to the trait default.
pub struct QuiltHandler;

impl ModHandler for QuiltHandler {
    fn mod_type(&self) -> ModType {
        ModType::Quilt
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["quilt.mod.json"]
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
    fn no_dep_extraction() {
        assert_eq!(QuiltHandler.extract_deps("{}").unwrap(), []);
    }
}
