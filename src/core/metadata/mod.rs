//! Parsed metadata for every supported mod and plugin loader.

use std::fmt;
use std::io::{self, Read, Seek};
use std::panic;
use std::path::Path;

use serde::Serialize;
use zipcrawl::ZipManager;

use crate::error::{Error, Result};

use super::REGISTRY;
use super::embedded;
use super::loaders::{bukkit, bungee, fabric, forge_legacy, forge_modern, paper, quilt, velocity};

/// Parsed mod/plugin metadata from a JAR file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ModMetadata {
    Fabric(Box<fabric::metadata::FabricModMetadata>),
    Quilt(Box<quilt::metadata::QuiltModMetadata>),
    ForgeLegacy(forge_legacy::metadata::ForgeLegacyMetadata),
    ForgeModern(Box<forge_modern::metadata::ModsTomlMetadata>),
    NeoForge(Box<forge_modern::metadata::ModsTomlMetadata>),
    Bukkit(bukkit::metadata::BukkitPluginMetadata),
    Bungee(bungee::metadata::BungeePluginMetadata),
    Paper(paper::metadata::PaperPluginMetadata),
    Velocity(velocity::metadata::VelocityPluginMetadata),
}

impl fmt::Display for ModMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fabric(m) => {
                writeln!(f, "Type:     Fabric")?;
                write!(f, "{m}")?;
            }
            Self::Quilt(m) => {
                writeln!(f, "Type:     Quilt")?;
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
            Self::Bungee(m) => {
                writeln!(f, "Type:     BungeeCord")?;
                write!(f, "{m}")?;
            }
            Self::Paper(m) => {
                writeln!(f, "Type:     Paper")?;
                write!(f, "{m}")?;
            }
            Self::Velocity(m) => {
                writeln!(f, "Type:     Velocity")?;
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
pub fn read_metadata_reader<R: Read>(reader: &mut R) -> Result<ModMetadata> {
    let mut mng = ZipManager::from_reader(reader)?;
    read_metadata_impl(&mut mng)
}

const MAX_METADATA_SIZE: usize = 10 * 1024 * 1024;

pub(crate) fn read_metadata_string<R: Read + Seek>(
    mng: &mut ZipManager<R>,
    name: &str,
) -> Result<String> {
    let file = mng.open_file(name)?;
    let mut buffer = Vec::new();
    let read = file
        .take((MAX_METADATA_SIZE as u64) + 1)
        .read_to_end(&mut buffer)
        .map_err(Error::Io)?;
    if read > MAX_METADATA_SIZE {
        return Err(Error::Usage(format!(
            "metadata file \"{name}\" exceeds maximum size limit of 10 MB"
        )));
    }
    String::from_utf8(buffer).map_err(|e| Error::Io(io::Error::new(io::ErrorKind::InvalidData, e)))
}

fn read_metadata_impl<R: Read + Seek>(mng: &mut ZipManager<R>) -> Result<ModMetadata> {
    let entries = mng.entries()?;
    let mod_type = REGISTRY.detect_archive(mng)?;

    let Some(file) = REGISTRY.find_metadata_file(&mod_type, &entries) else {
        return read_embedded(mng);
    };

    let raw = read_metadata_string(mng, file)?;
    let parsed = panic::catch_unwind(|| REGISTRY.read_metadata(&mod_type, &raw));
    match parsed {
        Ok(res) => res,
        Err(_) => Err(Error::UnsupportedMetadata(String::from(
            "malformed metadata caused parser panic",
        ))),
    }
}

/// Metadata of a shell jar, whose real mod sits inside an embedded one.
fn read_embedded<R: Read + Seek>(mng: &mut ZipManager<R>) -> Result<ModMetadata> {
    let Some((mod_type, raw)) = embedded::read_raw(mng)? else {
        return Err(Error::UnsupportedMetadata(mng.path_name.clone()));
    };

    let parsed = panic::catch_unwind(|| REGISTRY.read_metadata(&mod_type, &raw));
    match parsed {
        Ok(res) => res,
        Err(_) => Err(Error::UnsupportedMetadata(String::from(
            "malformed metadata caused parser panic",
        ))),
    }
}
