use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Parse a `paper-plugin.yml` string into a [`PaperPluginMetadata`].
///
/// # Errors
///
/// Returns [`crate::error::Error::Yaml`] if the input is not valid YAML.
pub fn parse(input: &str) -> Result<PaperPluginMetadata> {
    Ok(serde_saphyr::from_str(input)?)
}

/// Top-level `paper-plugin.yml` entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperPluginMetadata {
    pub name: String,
    pub version: String,
    pub main: String,

    #[serde(rename = "api-version")]
    pub api_version: Option<String>,

    #[serde(rename = "folia-supported")]
    pub folia_supported: Option<bool>,

    #[serde(default)]
    pub authors: Vec<String>,

    pub description: Option<String>,
    pub website: Option<String>,

    pub bootstrapper: Option<String>,
    pub loader: Option<String>,
    pub load: Option<String>,
    pub author: Option<String>,
}
