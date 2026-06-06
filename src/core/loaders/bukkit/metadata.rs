use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::Result;

pub fn parse(input: &str) -> Result<BukkitPluginMetadata> {
    Ok(serde_saphyr::from_str(input)?)
}

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

impl fmt::Display for BukkitPluginMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name:     {}", self.name)?;
        writeln!(f, "Version:  {}", self.version)?;
        if let Some(d) = &self.description {
            writeln!(f, "About:    {d}")?;
        }
        if !self.authors.is_empty() {
            writeln!(f, "Authors:  {}", self.authors.join(", "))?;
        }
        if !self.depend.is_empty() {
            writeln!(f, "Depends:  {}", self.depend.join(", "))?;
        }
        Ok(())
    }
}
