use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::Result;

pub fn parse(input: &str) -> Result<PaperPluginMetadata> {
    Ok(serde_saphyr::from_str(input)?)
}

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

impl fmt::Display for PaperPluginMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name:     {}", self.name)?;
        writeln!(f, "Version:  {}", self.version)?;
        if let Some(d) = &self.description {
            writeln!(f, "About:    {d}")?;
        }
        if !self.authors.is_empty() {
            writeln!(f, "Authors:  {}", self.authors.join(", "))?;
        }
        Ok(())
    }
}
