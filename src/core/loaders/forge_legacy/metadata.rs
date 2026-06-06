use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

pub fn parse(input: &str) -> Result<ForgeLegacyMetadata> {
    Ok(serde_json::from_str(input)?)
}

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

pub struct ForgeLegacyDisplay<'a>(pub &'a [ForgeLegacyModEntry]);

impl fmt::Display for ForgeLegacyModEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mod ID:   {}", self.modid)?;
        if let Some(n) = &self.name {
            writeln!(f, "Name:     {n}")?;
        }
        if let Some(d) = &self.description {
            writeln!(f, "About:    {d}")?;
        }
        if let Some(v) = &self.version {
            writeln!(f, "Version:  {v}")?;
        }
        if let Some(mv) = &self.mcversion {
            writeln!(f, "MC:       {mv}")?;
        }
        if !self.author_list.is_empty() {
            writeln!(f, "Authors:  {}", self.author_list.join(", "))?;
        }
        if !self.dependencies.is_empty() {
            writeln!(f, "Deps:     {}", self.dependencies.join(", "))?;
        }
        Ok(())
    }
}

impl fmt::Display for ForgeLegacyDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, e) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f, "---")?;
            }
            write!(f, "{e}")?;
        }
        Ok(())
    }
}
