//! Mod and plugin type enums.

use std::fmt;

use serde::Serialize;

/// Metadata format used by a Forge mod.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeModFormat {
    /// Modern Forge metadata stored in `META-INF/mods.toml`.
    ModsToml,
    /// Legacy Forge metadata stored in `mcmod.info`.
    McmodInfo,
}

impl fmt::Display for ForgeModFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModsToml => write!(f, "mods.toml"),
            Self::McmodInfo => write!(f, "mcmod.info"),
        }
    }
}

impl Serialize for ForgeModFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Server plugin platform detected from plugin metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginType {
    /// Bukkit or Spigot plugin (`plugin.yml`).
    Bukkit,
    /// Paper plugin (`paper-plugin.yml`).
    Paper,
    /// `BungeeCord` proxy plugin (`bungee.yml`).
    Bungee,
    /// Velocity proxy plugin (`velocity-plugin.json`).
    Velocity,
}

impl fmt::Display for PluginType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bukkit => write!(f, "Bukkit/Spigot"),
            Self::Paper => write!(f, "Paper"),
            Self::Bungee => write!(f, "BungeeCord"),
            Self::Velocity => write!(f, "Velocity"),
        }
    }
}

impl Serialize for PluginType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Kind of mod or plugin detected inside an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModType {
    /// Fabric mod (`fabric.mod.json`).
    Fabric,
    /// Quilt mod (`quilt.mod.json`).
    Quilt,
    /// Forge mod, including the metadata format that was detected.
    Forge(ForgeModFormat),
    /// `NeoForge` mod (`META-INF/neoforge.mods.toml`).
    NeoForge,
    /// Server plugin for the wrapped platform.
    Plugin(PluginType),
    /// No known loader metadata was found.
    Unknown,
}

impl fmt::Display for ModType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fabric => write!(f, "Fabric"),
            Self::Quilt => write!(f, "Quilt"),
            Self::NeoForge => write!(f, "NeoForge"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Forge(fmt) => write!(f, "Forge ({fmt})"),
            Self::Plugin(ty) => write!(f, "Plugin ({ty})"),
        }
    }
}

impl Serialize for ModType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forge_mod_format_display() {
        assert_eq!(ForgeModFormat::ModsToml.to_string(), "mods.toml");
        assert_eq!(ForgeModFormat::McmodInfo.to_string(), "mcmod.info");
    }

    #[test]
    fn plugin_type_display() {
        assert_eq!(PluginType::Bukkit.to_string(), "Bukkit/Spigot");
        assert_eq!(PluginType::Paper.to_string(), "Paper");
        assert_eq!(PluginType::Bungee.to_string(), "BungeeCord");
        assert_eq!(PluginType::Velocity.to_string(), "Velocity");
    }

    #[test]
    fn mod_type_fabric() {
        assert_eq!(ModType::Fabric.to_string(), "Fabric");
    }

    #[test]
    fn mod_type_quilt() {
        assert_eq!(ModType::Quilt.to_string(), "Quilt");
    }

    #[test]
    fn mod_type_neoforge() {
        assert_eq!(ModType::NeoForge.to_string(), "NeoForge");
    }

    #[test]
    fn mod_type_unknown() {
        assert_eq!(ModType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn mod_type_forge_modstoml() {
        assert_eq!(
            ModType::Forge(ForgeModFormat::ModsToml).to_string(),
            "Forge (mods.toml)"
        );
    }

    #[test]
    fn mod_type_forge_mcmodinfo() {
        assert_eq!(
            ModType::Forge(ForgeModFormat::McmodInfo).to_string(),
            "Forge (mcmod.info)"
        );
    }

    #[test]
    fn mod_type_plugin_bukkit() {
        assert_eq!(
            ModType::Plugin(PluginType::Bukkit).to_string(),
            "Plugin (Bukkit/Spigot)"
        );
    }

    #[test]
    fn mod_type_plugin_paper() {
        assert_eq!(
            ModType::Plugin(PluginType::Paper).to_string(),
            "Plugin (Paper)"
        );
    }

    #[test]
    fn mod_type_plugin_bungee() {
        assert_eq!(
            ModType::Plugin(PluginType::Bungee).to_string(),
            "Plugin (BungeeCord)"
        );
    }

    #[test]
    fn mod_type_plugin_velocity() {
        assert_eq!(
            ModType::Plugin(PluginType::Velocity).to_string(),
            "Plugin (Velocity)"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn mod_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ModType::Fabric).unwrap(),
            "\"Fabric\""
        );
        assert_eq!(
            serde_json::to_string(&ModType::Forge(ForgeModFormat::ModsToml)).unwrap(),
            "\"Forge (mods.toml)\""
        );
        assert_eq!(
            serde_json::to_string(&ModType::Plugin(PluginType::Paper)).unwrap(),
            "\"Plugin (Paper)\""
        );
        assert_eq!(
            serde_json::to_string(&ModType::Unknown).unwrap(),
            "\"Unknown\""
        );
    }
}
