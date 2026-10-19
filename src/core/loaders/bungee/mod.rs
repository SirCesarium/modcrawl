pub mod metadata;

use crate::core::detect_mod_type::{ModType, PluginType};
use crate::core::metadata::ModMetadata;
use crate::core::registry::ModHandler;
use crate::error::Result;

/// `BungeeCord` proxy plugins (`bungee.yml`). Metadata only; dependency
/// extraction is left to the trait default.
pub struct BungeeHandler;

impl ModHandler for BungeeHandler {
    fn mod_type(&self) -> ModType {
        ModType::Plugin(PluginType::Bungee)
    }

    fn detection_files(&self) -> &'static [&'static str] {
        &["bungee.yml"]
    }

    fn read_metadata(&self, raw: &str) -> Result<ModMetadata> {
        Ok(ModMetadata::Bungee(metadata::parse(raw)?))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn no_dep_extraction() {
        assert_eq!(BungeeHandler.extract_deps("").unwrap(), []);
    }
}
