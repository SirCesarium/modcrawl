use zipcrawl::{ZipEntry, ZipManager};

use crate::core::dep::types::DepEntry;
use crate::core::detect_mod_type::ModType;
use crate::core::metadata::ModMetadata;
use crate::error::Error;
use crate::error::Result;

/// Handler for a specific mod/plugin type.
pub trait ModHandler: Send + Sync {
    fn mod_type(&self) -> ModType;
    fn detection_files(&self) -> &'static [&'static str];

    /// Extract dependencies from a JAR.
    ///
    /// # Errors
    ///
    /// Returns an error if the JAR cannot be read or parsed.
    fn extract_deps(&self, mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
        let _ = mng;
        Ok(Vec::new())
    }

    /// Read metadata from raw file contents.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata cannot be parsed.
    fn read_metadata(&self, raw: &str) -> Result<ModMetadata>;

    fn metadata_file_path(&self) -> Option<&'static str> {
        self.detection_files().first().copied()
    }
}

/// Central registry of all known mod/plugin type handlers.
pub struct Registry {
    pub(crate) handlers: Vec<Box<dyn ModHandler>>,
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn register(&mut self, handler: Box<dyn ModHandler>) {
        self.handlers.push(handler);
    }

    #[must_use]
    pub fn detect(&self, entries: &[ZipEntry]) -> ModType {
        for handler in &self.handlers {
            for file in handler.detection_files() {
                if entries.iter().any(|e| e.name.as_str() == *file) {
                    return handler.mod_type();
                }
            }
        }
        ModType::Unknown
    }

    /// Find a handler by detection file path.
    #[must_use]
    #[allow(dead_code)]
    pub fn handler_by_file(&self, file: &str) -> Option<&dyn ModHandler> {
        self.handlers
            .iter()
            .find(|h| h.detection_files().contains(&file))
            .map(Box::as_ref)
    }

    #[must_use]
    pub fn handler(&self, mod_type: &ModType) -> Option<&dyn ModHandler> {
        self.handlers
            .iter()
            .find(|h| h.mod_type() == *mod_type)
            .map(Box::as_ref)
    }

    #[must_use]
    pub fn metadata_file_path(&self, mod_type: &ModType) -> Option<&'static str> {
        self.handler(mod_type)
            .and_then(ModHandler::metadata_file_path)
    }

    /// Extract dependencies for a given mod type.
    ///
    /// # Errors
    ///
    /// Returns an error if the JAR cannot be read or parsed.
    pub fn extract_deps(&self, mod_type: &ModType, mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
        match self.handler(mod_type) {
            Some(h) => h.extract_deps(mng),
            None => Ok(Vec::new()),
        }
    }

    /// Read metadata for a given mod type.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata format is unsupported or cannot be parsed.
    pub fn read_metadata(&self, mod_type: &ModType, raw: &str) -> Result<ModMetadata> {
        match self.handler(mod_type) {
            Some(h) => h.read_metadata(raw),
            None => Err(Error::UnsupportedMetadata(mod_type.to_string())),
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::core::dep::types::{DepEntry, DepKind, VersionRange};
    use crate::core::detect_mod_type::{ForgeModFormat, PluginType};

    struct MockFabricHandler;
    impl ModHandler for MockFabricHandler {
        fn mod_type(&self) -> ModType {
            ModType::Fabric
        }
        fn detection_files(&self) -> &'static [&'static str] {
            &["fabric.mod.json"]
        }
        fn read_metadata(&self, _raw: &str) -> Result<ModMetadata> {
            Err(Error::UnsupportedMetadata("mock".to_owned()))
        }
        fn extract_deps(&self, _mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
            Ok(vec![DepEntry::new(
                "mock-dep",
                DepKind::Required,
                VersionRange::parse(None),
            )])
        }
    }

    struct MockForgeHandler;
    impl ModHandler for MockForgeHandler {
        fn mod_type(&self) -> ModType {
            ModType::Forge(ForgeModFormat::ModsToml)
        }
        fn detection_files(&self) -> &'static [&'static str] {
            &["META-INF/mods.toml"]
        }
        fn read_metadata(&self, _raw: &str) -> Result<ModMetadata> {
            Err(Error::UnsupportedMetadata("mock".to_owned()))
        }
    }

    struct MockPaperHandler;
    impl ModHandler for MockPaperHandler {
        fn mod_type(&self) -> ModType {
            ModType::Plugin(PluginType::Paper)
        }
        fn detection_files(&self) -> &'static [&'static str] {
            &["paper-plugin.yml"]
        }
        fn read_metadata(&self, _raw: &str) -> Result<ModMetadata> {
            Err(Error::UnsupportedMetadata("mock".to_owned()))
        }
    }

    fn entry(name: &str) -> ZipEntry {
        ZipEntry {
            name: name.to_owned(),
            is_dir: false,
            size: 0,
            crc: 0,
        }
    }

    #[test]
    fn registry_detect_empty() {
        let r = Registry::new();
        let entries = vec![entry("fabric.mod.json")];
        assert_eq!(r.detect(&entries), ModType::Unknown);
    }

    #[test]
    fn registry_detect_single_match() {
        let mut r = Registry::new();
        r.register(Box::new(MockFabricHandler));
        let entries = vec![entry("fabric.mod.json")];
        assert_eq!(r.detect(&entries), ModType::Fabric);
    }

    #[test]
    fn registry_detect_first_wins() {
        let mut r = Registry::new();
        r.register(Box::new(MockPaperHandler));
        r.register(Box::new(MockForgeHandler));
        // Both paper-plugin.yml and META-INF/mods.toml present
        let entries = vec![entry("paper-plugin.yml"), entry("META-INF/mods.toml")];
        assert_eq!(r.detect(&entries), ModType::Plugin(PluginType::Paper));
    }

    #[test]
    fn registry_detect_no_match() {
        let mut r = Registry::new();
        r.register(Box::new(MockFabricHandler));
        let entries = vec![entry("some-random-file.txt")];
        assert_eq!(r.detect(&entries), ModType::Unknown);
    }

    #[test]
    fn registry_handler_found() {
        let mut r = Registry::new();
        r.register(Box::new(MockFabricHandler));
        assert!(r.handler(&ModType::Fabric).is_some());
    }

    #[test]
    fn registry_handler_not_found() {
        let r = Registry::new();
        assert!(r.handler(&ModType::Fabric).is_none());
    }

    #[test]
    fn registry_handler_by_file_found() {
        let mut r = Registry::new();
        r.register(Box::new(MockFabricHandler));
        assert!(r.handler_by_file("fabric.mod.json").is_some());
    }

    #[test]
    fn registry_handler_by_file_not_found() {
        let r = Registry::new();
        assert!(r.handler_by_file("fabric.mod.json").is_none());
    }

    #[test]
    fn registry_metadata_file_path_found() {
        let mut r = Registry::new();
        r.register(Box::new(MockFabricHandler));
        assert_eq!(
            r.metadata_file_path(&ModType::Fabric),
            Some("fabric.mod.json")
        );
    }

    #[test]
    fn registry_metadata_file_path_not_found() {
        let r = Registry::new();
        assert_eq!(r.metadata_file_path(&ModType::Fabric), None);
    }

    /// Minimal valid ZIP bytes (empty archive).
    fn empty_zip_bytes() -> Vec<u8> {
        vec![
            0x50, 0x4B, 0x05, 0x06, // EOCD signature
            0x00, 0x00, // disk number
            0x00, 0x00, // disk with central dir
            0x00, 0x00, // entries on disk
            0x00, 0x00, // total entries
            0x00, 0x00, 0x00, 0x00, // central dir size
            0x00, 0x00, 0x00, 0x00, // central dir offset
            0x00, 0x00, // comment length
        ]
    }

    fn make_mng() -> ZipManager {
        let mut cursor = Cursor::new(empty_zip_bytes());
        ZipManager::from_reader(&mut cursor).unwrap()
    }

    #[test]
    fn registry_extract_deps_found() {
        let mut r = Registry::new();
        r.register(Box::new(MockFabricHandler));
        let mut mng = make_mng();
        let deps = r.extract_deps(&ModType::Fabric, &mut mng).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "mock-dep");
    }

    #[test]
    fn registry_extract_deps_not_found() {
        let r = Registry::new();
        let mut mng = make_mng();
        let deps = r.extract_deps(&ModType::Fabric, &mut mng).unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn registry_read_metadata_found() {
        let mut r = Registry::new();
        r.register(Box::new(MockFabricHandler));
        let result = r.read_metadata(&ModType::Fabric, "{}");
        assert!(result.is_err());
    }

    #[test]
    fn registry_read_metadata_not_found() {
        let r = Registry::new();
        let result = r.read_metadata(&ModType::Fabric, "{}");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }
}
