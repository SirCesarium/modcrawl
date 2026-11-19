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

impl ModType {
    /// Stable machine identifier used in JSON output.
    #[must_use]
    pub fn id(&self) -> &'static str {
        match self {
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::Forge(ForgeModFormat::ModsToml) => "forge",
            Self::Forge(ForgeModFormat::McmodInfo) => "forge_legacy",
            Self::NeoForge => "neoforge",
            Self::Plugin(PluginType::Bukkit) => "bukkit",
            Self::Plugin(PluginType::Paper) => "paper",
            Self::Plugin(PluginType::Bungee) => "bungee",
            Self::Plugin(PluginType::Velocity) => "velocity",
            Self::Unknown => "unknown",
        }
    }

    /// Human-readable loader name shared by every text renderer.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fabric => "Fabric",
            Self::Quilt => "Quilt",
            Self::Forge(ForgeModFormat::ModsToml) => "Forge",
            Self::Forge(ForgeModFormat::McmodInfo) => "Forge (legacy)",
            Self::NeoForge => "NeoForge",
            Self::Plugin(PluginType::Bukkit) => "Bukkit",
            Self::Plugin(PluginType::Paper) => "Paper",
            Self::Plugin(PluginType::Bungee) => "Bungee",
            Self::Plugin(PluginType::Velocity) => "Velocity",
            Self::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for ModType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl Serialize for ModType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.id())
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
    fn mod_type_labels() {
        let cases = [
            (ModType::Fabric, "Fabric"),
            (ModType::Quilt, "Quilt"),
            (ModType::Forge(ForgeModFormat::ModsToml), "Forge"),
            (ModType::Forge(ForgeModFormat::McmodInfo), "Forge (legacy)"),
            (ModType::NeoForge, "NeoForge"),
            (ModType::Plugin(PluginType::Bukkit), "Bukkit"),
            (ModType::Plugin(PluginType::Paper), "Paper"),
            (ModType::Plugin(PluginType::Bungee), "Bungee"),
            (ModType::Plugin(PluginType::Velocity), "Velocity"),
            (ModType::Unknown, "Unknown"),
        ];

        for (kind, label) in cases {
            assert_eq!(kind.label(), label);
            assert_eq!(kind.to_string(), label);
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn mod_type_ids() {
        let cases = [
            (ModType::Fabric, "fabric"),
            (ModType::Quilt, "quilt"),
            (ModType::Forge(ForgeModFormat::ModsToml), "forge"),
            (ModType::Forge(ForgeModFormat::McmodInfo), "forge_legacy"),
            (ModType::NeoForge, "neoforge"),
            (ModType::Plugin(PluginType::Bukkit), "bukkit"),
            (ModType::Plugin(PluginType::Paper), "paper"),
            (ModType::Plugin(PluginType::Bungee), "bungee"),
            (ModType::Plugin(PluginType::Velocity), "velocity"),
            (ModType::Unknown, "unknown"),
        ];

        for (kind, id) in cases {
            assert_eq!(kind.id(), id);
            assert_eq!(serde_json::to_string(&kind).unwrap(), format!("\"{id}\""));
        }
    }
}
