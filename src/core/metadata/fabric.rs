use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

/// Parse a `fabric.mod.json` string into a [`FabricModMetadata`].
///
/// # Errors
///
/// Returns [`crate::error::Error::Json`] if the input is not valid JSON.
pub fn parse(input: &str) -> Result<FabricModMetadata> {
    Ok(serde_json::from_str(input)?)
}

/// Top-level `fabric.mod.json` entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabricModMetadata {
    #[serde(rename = "schemaVersion")]
    pub schema_version: i64,

    pub id: String,
    pub version: String,
    pub name: Option<String>,
    pub description: Option<String>,

    #[serde(default)]
    pub authors: Vec<Value>,

    pub contact: Option<FabricContact>,

    pub license: Option<String>,
    pub icon: Option<String>,
    pub environment: Option<String>,

    #[serde(default)]
    pub entrypoints: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub mixins: Vec<Value>,

    #[serde(default)]
    pub depends: HashMap<String, String>,

    #[serde(default)]
    pub recommends: HashMap<String, String>,

    #[serde(default)]
    pub suggests: HashMap<String, String>,

    #[serde(default)]
    pub breaks: HashMap<String, String>,

    #[serde(default)]
    pub conflicts: HashMap<String, String>,

    #[serde(default)]
    pub jars: Vec<FabricJarEntry>,

    #[serde(default)]
    pub custom: HashMap<String, Value>,

    #[serde(rename = "accessWidener")]
    pub access_widener: Option<String>,

    #[serde(default)]
    pub provides: Vec<String>,

    #[serde(default)]
    pub contributors: Vec<Value>,

    #[serde(default)]
    #[serde(rename = "languageAdapters")]
    pub language_adapters: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabricContact {
    pub sources: Option<String>,
    pub homepage: Option<String>,
    pub issues: Option<String>,
    pub discord: Option<String>,
    pub irc: Option<String>,
    #[serde(rename = "source")]
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabricJarEntry {
    pub file: String,
}
