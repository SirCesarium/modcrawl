//! Metadata types for `paper-plugin.yml`.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_saphyr::from_str;

use crate::error::Result;

/// Parses a Paper `paper-plugin.yml` document.
///
/// # Errors
///
/// Returns an error when the YAML is malformed or does not match the schema.
pub fn parse(input: &str) -> Result<PaperPluginMetadata> {
    Ok(from_str(input)?)
}

/// A dependency configuration declared in `dependencies`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PaperDependencyEntry {
    pub load: Option<String>,
    pub required: Option<bool>,
    #[serde(rename = "join-classpath")]
    pub join_classpath: Option<bool>,
}

/// The `dependencies` section of a Paper plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PaperDependencies {
    #[serde(default)]
    pub bootstrap: BTreeMap<String, Option<PaperDependencyEntry>>,
    #[serde(default)]
    pub server: BTreeMap<String, Option<PaperDependencyEntry>>,
}

/// Metadata declared by a Paper plugin in `paper-plugin.yml`.
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
    #[serde(default)]
    pub dependencies: Option<PaperDependencies>,
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
        if let Some(deps) = &self.dependencies {
            let mut dep_names = Vec::new();
            for (name, entry) in &deps.bootstrap {
                let is_opt = entry.as_ref().and_then(|e| e.required) == Some(false);
                if is_opt {
                    dep_names.push(format!("{name} (optional)"));
                } else {
                    dep_names.push(name.clone());
                }
            }
            for (name, entry) in &deps.server {
                let is_opt = entry.as_ref().and_then(|e| e.required) == Some(false);
                if is_opt {
                    dep_names.push(format!("{name} (optional)"));
                } else {
                    dep_names.push(name.clone());
                }
            }
            if !dep_names.is_empty() {
                writeln!(f, "Depends:  {}", dep_names.join(", "))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        assert_eq!(meta.dependencies, None);
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
bootstrapper: com.example.Bootstrap
loader: com.example.Loader
dependencies:
  bootstrap:
    BootMod:
      load: BEFORE
      required: true
  server:
    WorldEdit:
      load: BEFORE
      required: true
    PlaceholderAPI:
      required: false
"#;
        let meta = parse(yaml).unwrap();
        assert_eq!(meta.description.as_deref(), Some("A test plugin"));
        assert_eq!(meta.authors, vec!["Alice".to_string(), "Bob".to_string()]);
        assert_eq!(meta.bootstrapper.as_deref(), Some("com.example.Bootstrap"));
        assert_eq!(meta.loader.as_deref(), Some("com.example.Loader"));
        let deps = meta.dependencies.unwrap();
        assert!(deps.bootstrap.contains_key("BootMod"));
        assert!(deps.server.contains_key("WorldEdit"));
        assert!(deps.server.contains_key("PlaceholderAPI"));
    }

    #[test]
    fn display_output() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: com.example.TestPlugin
dependencies:
  server:
    WorldEdit:
      load: BEFORE
      required: true
    OptDep:
      required: false
"#;
        let meta = parse(yaml).unwrap();
        let out = meta.to_string();
        assert!(out.contains("Name:     TestPlugin"));
        assert!(out.contains("Version:  1.0.0"));
        assert!(out.contains("Depends:  OptDep (optional), WorldEdit"));
    }
}
