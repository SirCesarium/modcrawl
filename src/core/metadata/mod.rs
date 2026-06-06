use std::fmt;
use std::io::Read;
use std::path::Path;

use zipcrawl::ZipManager;

use crate::error::{Error, Result};

use super::loaders::{bukkit, fabric, forge_legacy, forge_modern, paper};
use super::REGISTRY;

/// Parsed mod/plugin metadata from a JAR file.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ModMetadata {
    Fabric(Box<fabric::metadata::FabricModMetadata>),
    ForgeLegacy(forge_legacy::metadata::ForgeLegacyMetadata),
    ForgeModern(Box<forge_modern::metadata::ModsTomlMetadata>),
    NeoForge(Box<forge_modern::metadata::ModsTomlMetadata>),
    Bukkit(bukkit::metadata::BukkitPluginMetadata),
    Paper(paper::metadata::PaperPluginMetadata),
}

impl fmt::Display for ModMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fabric(m) => {
                writeln!(f, "Type:     Fabric")?;
                write!(f, "{m}")?;
            }
            Self::ForgeLegacy(entries) => {
                writeln!(f, "Type:     Forge (mcmod.info)")?;
                write!(f, "{}", forge_legacy::metadata::ForgeLegacyDisplay(entries))?;
            }
            Self::ForgeModern(m) => {
                writeln!(f, "Type:     Forge (mods.toml)")?;
                write!(f, "{m}")?;
            }
            Self::NeoForge(m) => {
                writeln!(f, "Type:     NeoForge")?;
                write!(f, "{m}")?;
            }
            Self::Bukkit(m) => {
                writeln!(f, "Type:     Bukkit/Spigot")?;
                write!(f, "{m}")?;
            }
            Self::Paper(m) => {
                writeln!(f, "Type:     Paper")?;
                write!(f, "{m}")?;
            }
        }
        Ok(())
    }
}

/// Read and parse metadata from a JAR file on disk.
///
/// # Errors
///
/// Returns an error if the file cannot be read, is not a valid ZIP archive,
/// the metadata file is missing, or the metadata cannot be parsed.
pub fn read_metadata(path: &Path) -> Result<ModMetadata> {
    let mut mng = ZipManager::new(path)?;
    read_metadata_impl(&mut mng)
}

/// Read and parse metadata from a ZIP archive read via a `Read` impl.
///
/// # Errors
///
/// Returns an error if the data cannot be read, is not a valid ZIP archive,
/// the metadata file is missing, or the metadata cannot be parsed.
#[allow(dead_code)]
pub fn read_metadata_reader<R: Read>(reader: &mut R) -> Result<ModMetadata> {
    let mut mng = ZipManager::from_reader(reader)?;
    read_metadata_impl(&mut mng)
}

fn read_metadata_impl(mng: &mut ZipManager) -> Result<ModMetadata> {
    let entries = mng.entries()?;
    let mod_type = REGISTRY.detect(&entries);

    let Some(file) = REGISTRY.metadata_file_path(&mod_type) else {
        return Err(Error::UnsupportedMetadata(mng.path_name.clone()));
    };

    let raw = mng.read_to_string(file)?;
    REGISTRY.read_metadata(&mod_type, &raw)
}
