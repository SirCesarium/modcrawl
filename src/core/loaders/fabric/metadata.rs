use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

pub fn parse(input: &str) -> Result<FabricModMetadata> {
    Ok(serde_json::from_str(input)?)
}

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
    pub depends: HashMap<String, Value>,
    #[serde(default)]
    pub recommends: HashMap<String, Value>,
    #[serde(default)]
    pub suggests: HashMap<String, Value>,
    #[serde(default)]
    pub breaks: HashMap<String, Value>,
    #[serde(default)]
    pub conflicts: HashMap<String, Value>,
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

impl fmt::Display for FabricModMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ID:       {}", self.id)?;
        if let Some(n) = &self.name {
            writeln!(f, "Name:     {n}")?;
        }
        writeln!(f, "Version:  {}", self.version)?;
        if let Some(d) = &self.description {
            writeln!(f, "About:    {d}")?;
        }
        if let Some(l) = &self.license {
            writeln!(f, "License:  {l}")?;
        }
        if let Some(c) = &self.contact {
            if let Some(s) = &c.sources {
                writeln!(f, "Sources:  {s}")?;
            }
            if let Some(h) = &c.homepage {
                writeln!(f, "Homepage: {h}")?;
            }
        }
        if !self.depends.is_empty() {
            writeln!(f, "Depends:")?;
            for (k, v) in &self.depends {
                let ver = match v {
                    Value::String(s) => s.clone(),
                    Value::Array(arr) => arr
                        .iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect::<Vec<_>>()
                        .join(" | "),
                    other => other.to_string(),
                };
                writeln!(f, "  - {k} ({ver})")?;
            }
        }
        Ok(())
    }
}
