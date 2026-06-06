use std::fmt;
use std::path::Path;

use zipcrawl::ZipManager;

use crate::error::{Error, Result};

pub mod bukkit;
pub mod fabric;
pub mod forge_legacy;
pub mod forge_modern;
pub mod paper;

/// Parsed mod/plugin metadata from a JAR file.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ModMetadata {
    Fabric(Box<fabric::FabricModMetadata>),
    ForgeLegacy(forge_legacy::ForgeLegacyMetadata),
    ForgeModern(Box<forge_modern::ModsTomlMetadata>),
    NeoForge(Box<forge_modern::ModsTomlMetadata>),
    Bukkit(bukkit::BukkitPluginMetadata),
    Paper(paper::PaperPluginMetadata),
}

impl fmt::Display for ModMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fabric(m) => {
                writeln!(f, "Type:     Fabric")?;
                writeln!(f, "ID:       {}", m.id)?;
                if let Some(n) = &m.name {
                    writeln!(f, "Name:     {n}")?;
                }
                writeln!(f, "Version:  {}", m.version)?;
                if let Some(d) = &m.description {
                    writeln!(f, "About:    {d}")?;
                }
                if let Some(l) = &m.license {
                    writeln!(f, "License:  {l}")?;
                }
                if let Some(c) = &m.contact {
                    if let Some(s) = &c.sources {
                        writeln!(f, "Sources:  {s}")?;
                    }
                    if let Some(h) = &c.homepage {
                        writeln!(f, "Homepage: {h}")?;
                    }
                }
                if !m.depends.is_empty() {
                    writeln!(f, "Depends:")?;
                    for (k, v) in &m.depends {
                        writeln!(f, "  - {k} ({v})")?;
                    }
                }
            }
            Self::ForgeLegacy(entries) => {
                for (i, e) in entries.iter().enumerate() {
                    if i > 0 {
                        writeln!(f, "---")?;
                    }
                    writeln!(f, "Type:     Forge (mcmod.info)")?;
                    writeln!(f, "Mod ID:   {}", e.modid)?;
                    if let Some(n) = &e.name {
                        writeln!(f, "Name:     {n}")?;
                    }
                    if let Some(d) = &e.description {
                        writeln!(f, "About:    {d}")?;
                    }
                    if let Some(v) = &e.version {
                        writeln!(f, "Version:  {v}")?;
                    }
                    if let Some(mv) = &e.mcversion {
                        writeln!(f, "MC:       {mv}")?;
                    }
                    if !e.author_list.is_empty() {
                        writeln!(f, "Authors:  {}", e.author_list.join(", "))?;
                    }
                    if !e.dependencies.is_empty() {
                        writeln!(f, "Deps:     {}", e.dependencies.join(", "))?;
                    }
                }
            }
            Self::ForgeModern(m) | Self::NeoForge(m) => {
                let label = if matches!(self, Self::NeoForge(_)) {
                    "NeoForge"
                } else {
                    "Forge (mods.toml)"
                };
                writeln!(f, "Type:     {label}")?;
                if let Some(l) = &m.mod_loader {
                    writeln!(f, "Loader:   {l}")?;
                }
                if let Some(lv) = &m.loader_version {
                    writeln!(f, "LoaderV:  {lv}")?;
                }
                for mod_entry in &m.mods {
                    writeln!(f, "Mod ID:   {}", mod_entry.mod_id)?;
                    if let Some(n) = &mod_entry.display_name {
                        writeln!(f, "Name:     {n}")?;
                    }
                    if let Some(d) = &mod_entry.description {
                        writeln!(f, "About:    {d}")?;
                    }
                    if let Some(v) = &mod_entry.version {
                        writeln!(f, "Version:  {v}")?;
                    }
                    if let Some(a) = &mod_entry.authors {
                        writeln!(f, "Authors:  {a}")?;
                    }
                }
                if !m.dependencies.is_empty() {
                    writeln!(f, "Dependencies:")?;
                    for (_mod_id, deps) in &m.dependencies {
                        for dep in deps {
                            let vr = dep.version_range.as_deref().unwrap_or("*");
                            writeln!(f, "  - {} ({})", dep.mod_id, vr)?;
                        }
                    }
                }
            }
            Self::Bukkit(m) => {
                writeln!(f, "Type:     Bukkit/Spigot")?;
                writeln!(f, "Name:     {}", m.name)?;
                writeln!(f, "Version:  {}", m.version)?;
                if let Some(d) = &m.description {
                    writeln!(f, "About:    {d}")?;
                }
                if !m.authors.is_empty() {
                    writeln!(f, "Authors:  {}", m.authors.join(", "))?;
                }
                if !m.depend.is_empty() {
                    writeln!(f, "Depends:  {}", m.depend.join(", "))?;
                }
            }
            Self::Paper(m) => {
                writeln!(f, "Type:     Paper")?;
                writeln!(f, "Name:     {}", m.name)?;
                writeln!(f, "Version:  {}", m.version)?;
                if let Some(d) = &m.description {
                    writeln!(f, "About:    {d}")?;
                }
                if !m.authors.is_empty() {
                    writeln!(f, "Authors:  {}", m.authors.join(", "))?;
                }
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
    read_metadata_from(&mut mng, path)
}

/// Read and parse metadata from an already-open [`ZipManager`].
///
/// # Errors
///
/// Returns an error if the metadata file is missing or cannot be parsed.
fn read_metadata_from(mng: &mut ZipManager, path: &Path) -> Result<ModMetadata> {
    use super::detect_mod_type::{
        detect_mod_type, metadata_file_path, ForgeModFormat, ModType, PluginType,
    };

    let entries = mng.entries()?;
    let mod_type = detect_mod_type(&entries);

    let Some(file) = metadata_file_path(&mod_type) else {
        return Err(Error::UnsupportedMetadata(path.display().to_string()));
    };

    let raw = mng.read_to_string(file)?;

    match mod_type {
        ModType::Fabric => Ok(ModMetadata::Fabric(Box::new(fabric::parse(&raw)?))),
        ModType::Forge(ForgeModFormat::McmodInfo) => {
            Ok(ModMetadata::ForgeLegacy(forge_legacy::parse(&raw)?))
        }
        ModType::Forge(ForgeModFormat::ModsToml) => Ok(ModMetadata::ForgeModern(Box::new(
            forge_modern::parse(&raw)?,
        ))),
        ModType::NeoForge => Ok(ModMetadata::NeoForge(Box::new(forge_modern::parse(&raw)?))),
        ModType::Plugin(PluginType::Bukkit) => Ok(ModMetadata::Bukkit(bukkit::parse(&raw)?)),
        ModType::Plugin(PluginType::Paper) => Ok(ModMetadata::Paper(paper::parse(&raw)?)),
        _ => Err(Error::UnsupportedMetadata(path.display().to_string())),
    }
}
