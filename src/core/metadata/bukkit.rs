use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Parse a `plugin.yml` string into a [`BukkitPluginMetadata`].
///
/// # Errors
///
/// Returns [`crate::error::Error::Yaml`] if the input is not valid YAML.
pub fn parse(input: &str) -> Result<BukkitPluginMetadata> {
    Ok(serde_saphyr::from_str(input)?)
}

/// Top-level `plugin.yml` entry (Bukkit / Spigot).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BukkitPluginMetadata {
    pub name: String,
    pub version: String,
    pub main: String,

    #[serde(rename = "api-version")]
    pub api_version: Option<String>,

    #[serde(rename = "folia-supported")]
    pub folia_supported: Option<bool>,

    #[serde(default)]
    pub softdepend: Vec<String>,

    pub load: Option<String>,

    #[serde(default)]
    pub authors: Vec<String>,

    pub description: Option<String>,
    pub website: Option<String>,
    pub author: Option<String>,
    pub prefix: Option<String>,

    #[serde(default)]
    pub loadbefore: Vec<String>,

    #[serde(default)]
    pub depend: Vec<String>,
}
