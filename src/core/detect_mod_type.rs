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

/// A detection rule: if a ZIP entry with this file path exists, the mod type is determined.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DetectionRule {
    pub file: &'static str,
    pub mod_type: ModType,
}

/// Single source of truth for detection and metadata file paths.
///
/// Order matters: the first matching rule wins (Paper > Bukkit, NeoForge > Forge).
const DETECTION_ORDER: &[DetectionRule] = &[
    // Paper MUST be before Bukkit.
    DetectionRule {
        file: "paper-plugin.yml",
        mod_type: ModType::Plugin(PluginType::Paper),
    },
    DetectionRule {
        file: "plugin.yml",
        mod_type: ModType::Plugin(PluginType::Bukkit),
    },
    DetectionRule {
        file: "velocity-plugin.json",
        mod_type: ModType::Plugin(PluginType::Velocity),
    },
    DetectionRule {
        file: "bungee.yml",
        mod_type: ModType::Plugin(PluginType::Bungee),
    },
    // NeoForge MUST be before Forge.
    DetectionRule {
        file: "META-INF/neoforge.mods.toml",
        mod_type: ModType::NeoForge,
    },
    DetectionRule {
        file: "META-INF/mods.toml",
        mod_type: ModType::Forge(ForgeModFormat::ModsToml),
    },
    DetectionRule {
        file: "mcmod.info",
        mod_type: ModType::Forge(ForgeModFormat::McmodInfo),
    },
    DetectionRule {
        file: "fabric.mod.json",
        mod_type: ModType::Fabric,
    },
];

/// Detect the mod type from a list of ZIP entries.
#[must_use]
pub fn detect_mod_type(entries: &[ZipEntry]) -> ModType {
    for rule in DETECTION_ORDER {
        if entries.iter().any(|e| e.name.as_str() == rule.file) {
            return rule.mod_type.clone();
        }
    }
    ModType::Unknown
}

/// Returns the metadata file path inside the JAR for this mod type.
#[must_use]
pub fn metadata_file_path(mod_type: &ModType) -> Option<&'static str> {
    for rule in DETECTION_ORDER {
        if &rule.mod_type == mod_type {
            return Some(rule.file);
        }
    }
    None
}
