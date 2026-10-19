pub mod metadata;

use crate::core::detect_mod_type::{ModType, PluginType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// Velocity proxy plugins (`velocity-plugin.json`). Metadata only; dependency
/// extraction is left to the trait default.
pub struct VelocityHandler;

impl ModHandler for VelocityHandler {
    fn mod_type(&self) -> ModType {
        ModType::Plugin(PluginType::Velocity)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["velocity-plugin.json"]
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Velocity(metadata::parse(raw)?))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn no_dep_extraction() {
        assert!(VelocityHandler.extract_deps("{}").unwrap().is_empty());
    }
}
