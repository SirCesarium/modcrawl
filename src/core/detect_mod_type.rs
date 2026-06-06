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
}
