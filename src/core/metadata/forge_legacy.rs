use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

/// Parse an `mcmod.info` JSON string into a [`ForgeLegacyMetadata`].
///
/// # Errors
///
/// Returns [`crate::error::Error::Json`] if the input is not valid JSON.
pub fn parse(input: &str) -> Result<ForgeLegacyMetadata> {
    Ok(serde_json::from_str(input)?)
}

/// `mcmod.info` is a JSON array of mod entries.
pub type ForgeLegacyMetadata = Vec<ForgeLegacyModEntry>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForgeLegacyModEntry {
    pub modid: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub mcversion: Option<String>,
    pub url: Option<String>,

    #[serde(rename = "updateUrl")]
    pub update_url: Option<String>,

    #[serde(default)]
    #[serde(rename = "authorList")]
    pub author_list: Vec<String>,

    pub credits: Option<String>,

    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,

    #[serde(default)]
    pub screenshots: Vec<Value>,

    #[serde(default)]
    pub dependencies: Vec<String>,

    pub parent: Option<String>,

    #[serde(default)]
    pub authors: Vec<String>,
}
