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
