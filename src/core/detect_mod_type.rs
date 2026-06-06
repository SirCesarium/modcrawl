use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeModFormat {
    ModsToml,
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PluginType {
    Bukkit,
    Paper,
    Bungee,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModType {
    Fabric,
    Forge(ForgeModFormat),
    NeoForge,
    Plugin(PluginType),
    Unknown,
}

impl fmt::Display for ModType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fabric => write!(f, "Fabric"),
            Self::NeoForge => write!(f, "NeoForge"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Forge(fmt) => write!(f, "Forge ({fmt})"),
            Self::Plugin(ty) => write!(f, "Plugin ({ty})"),
        }
    }
}
