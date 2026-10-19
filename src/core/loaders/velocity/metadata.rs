use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::error::Result;

pub fn parse(input: &str) -> Result<VelocityPluginMetadata> {
    Ok(from_str(input)?)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VelocityPluginMetadata {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub main: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<VelocityDependency>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VelocityDependency {
    pub id: String,
    #[serde(default)]
    pub optional: bool,
}

impl fmt::Display for VelocityPluginMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ID:       {}", self.id)?;
        if let Some(n) = &self.name {
            writeln!(f, "Name:     {n}")?;
        }
        if let Some(v) = &self.version {
            writeln!(f, "Version:  {v}")?;
        }
        if let Some(m) = &self.main {
            writeln!(f, "Main:     {m}")?;
        }
        if let Some(d) = &self.description {
            writeln!(f, "About:    {d}")?;
        }
        if !self.authors.is_empty() {
            writeln!(f, "Authors:  {}", self.authors.join(", "))?;
        }
        if !self.dependencies.is_empty() {
            let deps: Vec<String> = self
                .dependencies
                .iter()
                .map(|d| {
                    if d.optional {
                        format!("{} (optional)", d.id)
                    } else {
                        d.id.clone()
                    }
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
        let json = r#"{"id":"myplugin","main":"com.example.MyPlugin"}"#;
        let meta = parse(json).unwrap();
        assert_eq!(meta.id, "myplugin");
        assert_eq!(meta.main.as_deref(), Some("com.example.MyPlugin"));
        assert!(meta.dependencies.is_empty());
    }

    #[test]
    fn full() {
        let json = r#"{
            "id": "plasmovoice",
            "name": "PlasmoVoice",
            "version": "2.2.0",
            "main": "su.plo.voice.velocity.VelocityVoiceProxy",
            "authors": ["Apehum"],
            "description": "A voice chat plugin",
            "dependencies": [
                {"id": "luckperms", "optional": true},
                {"id": "required_dep"}
            ]
        }"#;
        let meta = parse(json).unwrap();
        assert_eq!(meta.name.as_deref(), Some("PlasmoVoice"));
        assert_eq!(meta.version.as_deref(), Some("2.2.0"));
        assert_eq!(meta.authors, ["Apehum"]);
        assert_eq!(meta.dependencies.len(), 2);
        assert!(meta.dependencies[0].optional);
        assert!(!meta.dependencies[1].optional);
    }

    #[test]
    fn display_output() {
        let json = r#"{
            "id": "myplugin",
            "name": "MyPlugin",
            "version": "1.0.0",
            "authors": ["Alice"],
            "dependencies": [{"id": "luckperms", "optional": true}]
        }"#;
        let out = parse(json).unwrap().to_string();
        assert!(out.contains("ID:       myplugin"));
        assert!(out.contains("Name:     MyPlugin"));
        assert!(out.contains("Depends:  luckperms (optional)"));
    }
}
