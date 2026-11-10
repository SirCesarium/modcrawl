//! Parsed metadata for every supported mod and plugin loader.

use std::fmt;
use std::io::{self, Read, Seek};
use std::panic;
use std::path::Path;
use std::result::Result as StdResult;

use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};
use zipcrawl::ZipManager;

use crate::core::detect_mod_type::{ForgeModFormat, ModType, PluginType};
use crate::error::{Error, Result};

use super::REGISTRY;
use super::embedded;
use super::loaders::{bukkit, bungee, fabric, forge_legacy, forge_modern, paper, quilt, velocity};

/// Parsed mod/plugin metadata from a JAR file.
///
/// Serializes as a flat JSON object with a `"type"` field that identifies the
/// loader, followed by the loader-specific fields at the top level.
/// For example: `{"type": "Fabric", "id": "my-mod", ...}`.
///
/// `ForgeLegacy` is an exception: its inner value is a list of mod entries, so
/// it serializes as `{"type": "ForgeLegacy", "mods": [...]}`.
#[derive(Debug, Clone, PartialEq)]
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

impl Serialize for ModMetadata {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        /// Merge a `"type"` key with the fields of a serializable struct into a
        /// single flat map.  The inner value must serialize as a JSON object.
        fn flatten_with_type<S, T>(
            type_name: &'static str,
            inner: &T,
            serializer: S,
        ) -> StdResult<S::Ok, S::Error>
        where
            S: Serializer,
            T: Serialize,
        {
            use serde::ser::Error as _;
            let inner_val =
                serde_json::to_value(inner).map_err(|e| S::Error::custom(e.to_string()))?;
            let obj = inner_val
                .as_object()
                .ok_or_else(|| S::Error::custom("inner metadata did not serialize as an object"))?;
            let mut map = serializer.serialize_map(Some(obj.len() + 1))?;
            map.serialize_entry("type", type_name)?;
            for (k, v) in obj {
                map.serialize_entry(k, v)?;
            }
            map.end()
        }

        match self {
            Self::Fabric(m) => flatten_with_type("Fabric", m, serializer),
            Self::Quilt(m) => flatten_with_type("Quilt", m, serializer),
            Self::ForgeModern(m) => flatten_with_type("ForgeModern", m, serializer),
            Self::NeoForge(m) => flatten_with_type("NeoForge", m, serializer),
            Self::Bukkit(m) => flatten_with_type("Bukkit", m, serializer),
            Self::Bungee(m) => flatten_with_type("Bungee", m, serializer),
            Self::Paper(m) => flatten_with_type("Paper", m, serializer),
            Self::Velocity(m) => flatten_with_type("Velocity", m, serializer),
            // ForgeLegacy inner value is a Vec, not a struct, so it cannot be
            // flattened field-by-field; wrap it under a "mods" key instead.
            Self::ForgeLegacy(entries) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "ForgeLegacy")?;
                map.serialize_entry("mods", entries)?;
                map.end()
            }
        }
    }
}

impl ModMetadata {
    /// Returns the mod or plugin loader type for this metadata.
    #[must_use]
    pub fn mod_type(&self) -> ModType {
        match self {
            Self::Fabric(_) => ModType::Fabric,
            Self::Quilt(_) => ModType::Quilt,
            Self::ForgeLegacy(_) => ModType::Forge(ForgeModFormat::McmodInfo),
            Self::ForgeModern(_) => ModType::Forge(ForgeModFormat::ModsToml),
            Self::NeoForge(_) => ModType::NeoForge,
            Self::Bukkit(_) => ModType::Plugin(PluginType::Bukkit),
            Self::Bungee(_) => ModType::Plugin(PluginType::Bungee),
            Self::Paper(_) => ModType::Plugin(PluginType::Paper),
            Self::Velocity(_) => ModType::Plugin(PluginType::Velocity),
        }
    }
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

pub(crate) fn read_metadata_impl<R: Read + Seek>(mng: &mut ZipManager<R>) -> Result<ModMetadata> {
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
