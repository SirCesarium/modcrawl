//! Metadata types for `bungee.yml`.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_saphyr::from_str;

use crate::error::Result;

/// Parses a `BungeeCord` `bungee.yml` document.
///
/// # Errors
///
/// Returns an error when the YAML is malformed or does not match the schema.
pub fn parse(input: &str) -> Result<BungeePluginMetadata> {
    Ok(from_str(input)?)
}

/// Metadata declared by a `BungeeCord` plugin in `bungee.yml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BungeePluginMetadata {
    pub name: String,
    pub version: String,
    pub main: String,
    pub author: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(
        default,
        rename = "softDepends",
        alias = "softdepends",
        alias = "soft_depends",
        alias = "softdepend"
    )]
    pub soft_depends: Vec<String>,
}

impl fmt::Display for BungeePluginMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name:     {}", self.name)?;
        writeln!(f, "Version:  {}", self.version)?;
        writeln!(f, "Main:     {}", self.main)?;
        if let Some(d) = &self.description {
            writeln!(f, "About:    {d}")?;
        }
        let authors = if self.authors.is_empty() {
            self.author.clone().unwrap_or_default()
        } else {
            self.authors.join(", ")
        };
        if !authors.is_empty() {
            writeln!(f, "Authors:  {authors}")?;
        }
        if !self.depends.is_empty() {
            writeln!(f, "Depends:  {}", self.depends.join(", "))?;
        }
        if !self.soft_depends.is_empty() {
            writeln!(f, "Soft-Depends: {}", self.soft_depends.join(", "))?;
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
main: me.example.TestPlugin
"#;
        let meta = parse(yaml).unwrap();
        assert_eq!(meta.name, "TestPlugin");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.main, "me.example.TestPlugin");
        assert_eq!(meta.depends, Vec::<String>::new());
    }

    #[test]
    fn full() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: me.example.TestPlugin
description: A test plugin
author: Alice
authors: [Alice, Bob]
depends: [Dependency, Other]
softDepends: [SoftDep, OtherSoft]
"#;
        let meta = parse(yaml).unwrap();
        assert_eq!(meta.description.as_deref(), Some("A test plugin"));
        assert_eq!(meta.author.as_deref(), Some("Alice"));
        assert_eq!(meta.authors, ["Alice", "Bob"]);
        assert_eq!(meta.depends, ["Dependency", "Other"]);
        assert_eq!(meta.soft_depends, ["SoftDep", "OtherSoft"]);
    }

    #[test]
    fn softdepends_alias() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: me.example.TestPlugin
softdepends: [SoftDep]
"#;
        let meta = parse(yaml).unwrap();
        assert_eq!(meta.soft_depends, ["SoftDep"]);
    }

    #[test]
    fn display_output() {
        let yaml = r#"
name: TestPlugin
version: "1.0.0"
main: me.example.TestPlugin
authors: [Alice]
depends: [Dependency]
softDepends: [OptionalDep]
"#;
        let out = parse(yaml).unwrap().to_string();
        assert!(out.contains("Name:     TestPlugin"));
        assert!(out.contains("Main:     me.example.TestPlugin"));
        assert!(out.contains("Depends:  Dependency"));
        assert!(out.contains("Soft-Depends: OptionalDep"));
    }
}
