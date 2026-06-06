use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

/// Parse a `META-INF/mods.toml` string into a [`ModsTomlMetadata`].
///
/// # Errors
///
/// Returns [`crate::error::Error::Toml`] if the input is not valid TOML.
pub fn parse(input: &str) -> Result<ModsTomlMetadata> {
    Ok(toml::from_str(input)?)
}

/// Top-level `META-INF/mods.toml` entry (Forge / `NeoForge`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModsTomlMetadata {
    #[serde(rename = "modLoader")]
    pub mod_loader: Option<String>,

    #[serde(rename = "loaderVersion")]
    pub loader_version: Option<String>,

    #[serde(rename = "issueTrackerURL")]
    pub issue_tracker_url: Option<String>,

    pub license: Option<String>,

    #[serde(default)]
    pub mods: Vec<ModsTomlMod>,

    #[serde(default)]
    pub dependencies: HashMap<String, Vec<ModsTomlDependency>>,

    #[serde(rename = "displayURL")]
    pub display_url: Option<String>,

    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,

    pub authors: Option<String>,
    pub credits: Option<String>,

    #[serde(rename = "showAsResourcePack")]
    pub show_as_resource_pack: Option<bool>,

    #[serde(rename = "clientSideOnly")]
    pub client_side_only: Option<bool>,

    #[serde(default)]
    pub mixins: Vec<ModsTomlMixin>,

    #[serde(default)]
    pub modproperties: HashMap<String, HashMap<String, String>>,

    #[serde(rename = "enumExtensions")]
    pub enum_extensions: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModsTomlMod {
    #[serde(rename = "modId")]
    pub mod_id: String,

    pub version: Option<String>,

    #[serde(rename = "displayName")]
    pub display_name: Option<String>,

    #[serde(rename = "displayURL")]
    pub display_url: Option<String>,

    pub authors: Option<Value>,
    pub description: Option<String>,

    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,

    pub license: Option<String>,
    pub credits: Option<String>,

    #[serde(rename = "updateJSONURL")]
    pub update_jsonurl: Option<String>,

    #[serde(rename = "displayTest")]
    pub display_test: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModsTomlDependency {
    #[serde(rename = "modId")]
    pub mod_id: String,

    pub mandatory: Option<bool>,

    #[serde(rename = "versionRange")]
    pub version_range: Option<String>,

    pub ordering: Option<String>,
    pub side: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModsTomlMixin {
    pub config: String,
}
