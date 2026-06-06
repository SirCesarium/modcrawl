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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: com.example.TestPlugin
"#;
        let meta = parse(yaml).unwrap();
        assert_eq!(meta.name, "TestPlugin");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.main, "com.example.TestPlugin");
    }

    #[test]
    fn full() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: com.example.TestPlugin
description: A test plugin
author: Alice
authors: [Alice, Bob]
website: https://example.com
api-version: "1.20"
folia-supported: true
load: STARTUP
depend: [Vault, WorldEdit]
softdepend: [PlaceholderAPI]
loadbefore: [OtherPlugin]
prefix: Test
"#;
        let meta = parse(yaml).unwrap();
        assert_eq!(meta.description.as_deref(), Some("A test plugin"));
        assert_eq!(meta.authors, vec!["Alice".to_string(), "Bob".to_string()]);
        assert_eq!(
            meta.depend,
            vec!["Vault".to_string(), "WorldEdit".to_string()]
        );
        assert_eq!(meta.softdepend, vec!["PlaceholderAPI".to_string()]);
        assert_eq!(meta.loadbefore, vec!["OtherPlugin".to_string()]);
        assert!(meta.folia_supported == Some(true));
    }

    #[test]
    fn display_output() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: com.example.TestPlugin
"#;
        let meta = parse(yaml).unwrap();
        let out = meta.to_string();
        assert!(out.contains("Name:     TestPlugin"));
        assert!(out.contains("Version:  1.0.0"));
    }
}
