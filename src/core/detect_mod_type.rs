use core::fmt;

use zipcrawl::ZipEntry;

/*
   FORGE MOD FORMAT
*/

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

/*
   PLUGIN TYPE
*/

#[derive(Debug, Clone, PartialEq, Eq)]
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

/*
   MOD TYPE
*/

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModType {
    Fabric,
    Forge { forge_format: ForgeModFormat },
    NeoForge,
    Plugin { plugin_type: PluginType },
    Unknown,
}

impl fmt::Display for ModType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fabric => write!(f, "Fabric"),
            Self::NeoForge => write!(f, "NeoForge"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Forge { forge_format } => write!(f, "Forge ({forge_format})"),
            Self::Plugin { plugin_type } => write!(f, "Plugin ({plugin_type})"),
        }
    }
}

#[must_use]
pub fn detect_mod_type(entries: &[ZipEntry]) -> ModType {
    let has_file = |name: &str| entries.iter().any(|e| e.name.as_str() == name);

    /*
     PLUGINS

     ---

     PAPER MUST BE BEFORE BUKKIT.
    */
    if has_file("paper-plugin.yml") {
        return ModType::Plugin {
            plugin_type: PluginType::Paper,
        };
    }
    if has_file("plugin.yml") {
        return ModType::Plugin {
            plugin_type: PluginType::Bukkit,
        };
    }

    /*
    PROXIES
    */
    if has_file("velocity-plugin.json") {
        return ModType::Plugin {
            plugin_type: PluginType::Velocity,
        };
    }
    if has_file("bungee.yml") {
        return ModType::Plugin {
            plugin_type: PluginType::Bungee,
        };
    }

    /*
     MODS

     ---

     NeoForge MUST be before Forge.
    */
    if has_file("META-INF/neoforge.mods.toml") {
        return ModType::NeoForge;
    }
    if has_file("META-INF/mods.toml") {
        return ModType::Forge {
            forge_format: ForgeModFormat::ModsToml,
        };
    }
    if has_file("mcmod.info") {
        return ModType::Forge {
            forge_format: ForgeModFormat::McmodInfo,
        };
    }
    if has_file("fabric.mod.json") {
        return ModType::Fabric;
    }

    ModType::Unknown
}
