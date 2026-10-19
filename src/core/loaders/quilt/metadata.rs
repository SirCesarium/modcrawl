use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::error::Result;

pub fn parse(input: &str) -> Result<QuiltModMetadata> {
    Ok(from_str(input)?)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuiltModMetadata {
    #[serde(rename = "schema_version")]
    pub schema_version: Option<i64>,
    pub quilt_loader: QuiltLoader,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuiltLoader {
    pub id: String,
    pub version: Option<String>,
    pub group: Option<String>,
    #[serde(default)]
    pub metadata: QuiltMetadataInfo,
    #[serde(default)]
    pub depends: Vec<QuiltDep>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct QuiltMetadataInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    /// Contributor name mapped to their role. A `BTreeMap` keeps rendered and
    /// serialized output deterministic.
    #[serde(default)]
    pub contributors: BTreeMap<String, String>,
    pub contact: Option<QuiltContact>,
    pub license: Option<StringOrList>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuiltContact {
    pub homepage: Option<String>,
    pub issues: Option<String>,
    pub sources: Option<String>,
}

/// A `quilt.mod.json` dependency: either a bare mod id or an object with an
/// id and optional version constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QuiltDep {
    Bare(String),
    Detailed {
        id: String,
        #[serde(default)]
        versions: Option<QuiltVersions>,
    },
}

impl QuiltDep {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Bare(id) | Self::Detailed { id, .. } => id,
        }
    }

    #[must_use]
    pub fn versions(&self) -> Option<&QuiltVersions> {
        match self {
            Self::Bare(_) => None,
            Self::Detailed { versions, .. } => versions.as_ref(),
        }
    }
}

/// Version constraints for a Quilt dependency: a single string, a list of
/// strings, or the object form whose `all` key lists the required ranges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QuiltVersions {
    One(String),
    Many(Vec<String>),
    All { all: Vec<String> },
}

impl QuiltVersions {
    #[must_use]
    pub fn join(&self) -> String {
        match self {
            Self::One(s) => s.clone(),
            Self::Many(v) => v.join(" | "),
            Self::All { all } => all.join(" | "),
        }
    }
}

/// A plain string or a list of strings; `quilt.mod.json` accepts either for
/// `license` and dependency `versions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrList {
    One(String),
    Many(Vec<String>),
}

impl StringOrList {
    #[must_use]
    pub fn join(&self) -> String {
        match self {
            Self::One(s) => s.clone(),
            Self::Many(v) => v.join(" | "),
        }
    }
}

impl fmt::Display for QuiltModMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let loader = &self.quilt_loader;
        let meta = &loader.metadata;

        writeln!(f, "ID:       {}", loader.id)?;
        if let Some(n) = &meta.name {
            writeln!(f, "Name:     {n}")?;
        }
        if let Some(v) = &loader.version {
            writeln!(f, "Version:  {v}")?;
        }
        if let Some(d) = &meta.description {
            writeln!(f, "About:    {d}")?;
        }
        if let Some(l) = &meta.license {
            writeln!(f, "License:  {}", l.join())?;
        }
        if !meta.contributors.is_empty() {
            let contributors: Vec<String> = meta
                .contributors
                .iter()
                .map(|(name, role)| {
                    if role.is_empty() {
                        name.clone()
                    } else {
                        format!("{name} ({role})")
                    }
                })
                .collect();
            writeln!(f, "Contributors: {}", contributors.join(", "))?;
        }
        if let Some(c) = &meta.contact {
            if let Some(s) = &c.sources {
                writeln!(f, "Sources:  {s}")?;
            }
            if let Some(h) = &c.homepage {
                writeln!(f, "Homepage: {h}")?;
            }
        }
        if !loader.depends.is_empty() {
            let deps: Vec<String> = loader
                .depends
                .iter()
                .map(|d| match d.versions() {
                    Some(v) => format!("{} ({})", d.id(), v.join()),
                    None => d.id().to_owned(),
                })
                .collect();
            writeln!(f, "Depends:  {}", deps.join(", "))?;
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
        let json = r#"{
            "schema_version": 1,
            "quilt_loader": {"id": "example_mod", "version": "1.0.0"}
        }"#;
        let meta = parse(json).unwrap();
        assert_eq!(meta.schema_version, Some(1));
        assert_eq!(meta.quilt_loader.id, "example_mod");
        assert_eq!(meta.quilt_loader.version.as_deref(), Some("1.0.0"));
        assert_eq!(meta.quilt_loader.depends, []);
    }

    #[test]
    fn full() {
        let json = r#"{
            "schema_version": 1,
            "quilt_loader": {
                "group": "com.example",
                "id": "example_mod",
                "version": "1.0.0",
                "metadata": {
                    "name": "Mod Name",
                    "description": "A short description",
                    "contributors": {"Alice": "Owner", "Bob": "Contributor"},
                    "contact": {
                        "homepage": "https://example.com/",
                        "sources": "https://github.com/example/mod"
                    },
                    "license": ["MIT", "Apache-2.0"]
                },
                "depends": [
                    {"id": "quilt_loader", "versions": ">=0.19.1"},
                    {"id": "minecraft", "versions": [">=1.20", "<1.21"]},
                    {"id": "java", "versions": {"all": [">=17"]}},
                    "bare_dep"
                ]
            }
        }"#;
        let meta = parse(json).unwrap();
        let loader = &meta.quilt_loader;
        assert_eq!(loader.group.as_deref(), Some("com.example"));
        assert_eq!(loader.metadata.name.as_deref(), Some("Mod Name"));
        assert_eq!(loader.metadata.contributors["Alice"], "Owner");
        let license = loader.metadata.license.as_ref().unwrap();
        assert_eq!(license.join(), "MIT | Apache-2.0");
        assert_eq!(loader.depends.len(), 4);
        assert_eq!(
            loader.depends[1].versions().unwrap().join(),
            ">=1.20 | <1.21"
        );
        assert_eq!(loader.depends[2].versions().unwrap().join(), ">=17");
        assert_eq!(loader.depends[3].id(), "bare_dep");
        assert!(loader.depends[3].versions().is_none());
    }

    #[test]
    fn display_output() {
        let json = r#"{
            "schema_version": 1,
            "quilt_loader": {
                "id": "example_mod",
                "version": "1.0.0",
                "metadata": {
                    "name": "Mod Name",
                    "contributors": {"Alice": "Owner"},
                    "license": "MIT"
                },
                "depends": [{"id": "quilt_loader", "versions": ">=0.19.1"}]
            }
        }"#;
        let out = parse(json).unwrap().to_string();
        assert!(out.contains("ID:       example_mod"));
        assert!(out.contains("Name:     Mod Name"));
        assert!(out.contains("License:  MIT"));
        assert!(out.contains("Contributors: Alice (Owner)"));
        assert!(out.contains("Depends:  quilt_loader (>=0.19.1)"));
    }
}
