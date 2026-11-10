//! Metadata types for `fabric.mod.json`.

use std::collections::BTreeMap;
use std::fmt::{self, Write};
use std::result::Result as StdResult;

use serde::de::{Error as DeError, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, from_str};

use crate::error::Result;

/// Parses a `fabric.mod.json` document.
///
/// Mod authors sometimes write multi-line descriptions with literal newlines or
/// tabs in `fabric.mod.json`, which standard JSON disallows. When standard
/// parsing fails, control characters within string literals are sanitized and
/// parsing is retried.
///
/// # Errors
///
/// Returns an error when the JSON is malformed or does not match the schema.
pub fn parse(input: &str) -> Result<FabricModMetadata> {
    match from_str::<FabricModMetadata>(input) {
        Ok(meta) => Ok(meta),
        Err(e) => {
            let sanitized = sanitize_control_characters(input);
            if sanitized == input {
                Err(e.into())
            } else {
                Ok(from_str::<FabricModMetadata>(&sanitized)?)
            }
        }
    }
}

fn sanitize_control_characters(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if in_string {
            if escape {
                escape = false;
                out.push(ch);
            } else if ch == '\\' {
                escape = true;
                out.push(ch);
            } else if ch == '"' {
                in_string = false;
                out.push(ch);
            } else {
                match ch {
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if u32::from(c) < 0x20 => {
                        let _ = write!(out, "\\u{:04x}", u32::from(c));
                    }
                    _ => out.push(ch),
                }
            }
        } else {
            if ch == '"' {
                in_string = true;
            }
            out.push(ch);
        }
    }
    out
}

fn deserialize_license<'de, D>(deserializer: D) -> StdResult<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct LicenseVisitor;

    impl<'de> Visitor<'de> for LicenseVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a license string or an array of license strings")
        }

        fn visit_str<E>(self, value: &str) -> StdResult<Self::Value, E>
        where
            E: DeError,
        {
            Ok(Some(value.to_owned()))
        }

        fn visit_seq<S>(self, mut seq: S) -> StdResult<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut licenses = Vec::new();
            while let Some(license) = seq.next_element::<String>()? {
                licenses.push(license);
            }
            if licenses.is_empty() {
                Ok(None)
            } else {
                Ok(Some(licenses.join(", ")))
            }
        }

        fn visit_none<E>(self) -> StdResult<Self::Value, E>
        where
            E: DeError,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> StdResult<Self::Value, E>
        where
            E: DeError,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(LicenseVisitor)
}

/// Metadata declared by a Fabric mod in `fabric.mod.json`.
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
    #[serde(default, deserialize_with = "deserialize_license")]
    pub license: Option<String>,
    pub icon: Option<String>,
    pub environment: Option<String>,
    #[serde(default)]
    pub entrypoints: BTreeMap<String, Value>,
    #[serde(default)]
    pub mixins: Vec<Value>,
    #[serde(default)]
    pub depends: BTreeMap<String, Value>,
    #[serde(default)]
    pub recommends: BTreeMap<String, Value>,
    #[serde(default)]
    pub suggests: BTreeMap<String, Value>,
    #[serde(default)]
    pub breaks: BTreeMap<String, Value>,
    #[serde(default)]
    pub conflicts: BTreeMap<String, Value>,
    #[serde(default)]
    pub jars: Vec<FabricJarEntry>,
    #[serde(default)]
    pub custom: BTreeMap<String, Value>,
    #[serde(rename = "accessWidener")]
    pub access_widener: Option<String>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub contributors: Vec<Value>,
    #[serde(default)]
    #[serde(rename = "languageAdapters")]
    pub language_adapters: BTreeMap<String, String>,
}

/// Contact links declared by a Fabric mod.
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

/// An embedded JAR declared in a Fabric mod's `jars` list.
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn minimal() {
        let json = r#"{"schemaVersion": 1, "id": "testmod", "version": "1.0.0"}"#;
        let meta = parse(json).unwrap();
        assert_eq!(meta.id, "testmod");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.schema_version, 1);
    }

    #[test]
    fn full() {
        let json = r#"{
            "schemaVersion": 1,
            "id": "testmod",
            "version": "1.0.0",
            "name": "Test Mod",
            "description": "A test mod",
            "authors": ["Alice", "Bob"],
            "license": "MIT",
            "depends": {
                "fabric-api": ">=0.50.0"
            },
            "recommends": {
                "sodium": "*"
            }
        }"#;
        let meta = parse(json).unwrap();
        assert_eq!(meta.name.as_deref(), Some("Test Mod"));
        assert_eq!(meta.description.as_deref(), Some("A test mod"));
        assert_eq!(meta.license.as_deref(), Some("MIT"));
        assert!(meta.depends.contains_key("fabric-api"));
        assert!(meta.recommends.contains_key("sodium"));
    }

    #[test]
    fn display_output() {
        let json = r#"{"schemaVersion": 1, "id": "testmod", "version": "1.0.0", "name": "Test"}"#;
        let meta = parse(json).unwrap();
        let out = meta.to_string();
        assert!(out.contains("ID:       testmod"));
        assert!(out.contains("Name:     Test"));
        assert!(out.contains("Version:  1.0.0"));
    }

    #[test]
    fn license_array_deserializes() {
        let json = r#"{
            "schemaVersion": 1,
            "id": "testmod",
            "version": "1.0.0",
            "license": ["MIT", "Apache-2.0"]
        }"#;
        let meta = parse(json).unwrap();
        assert_eq!(meta.license.as_deref(), Some("MIT, Apache-2.0"));
    }

    #[test]
    fn entrypoints_with_objects() {
        let json = r#"{
            "schemaVersion": 1,
            "id": "testmod",
            "version": "1.0.0",
            "entrypoints": {
                "main": ["com.example.Mod::init"],
                "client": [{"adapter": "kotlin", "value": "com.example.Client::init"}]
            }
        }"#;
        let meta = parse(json).unwrap();
        assert!(meta.entrypoints.contains_key("main"));
        assert!(meta.entrypoints.contains_key("client"));
    }

    #[test]
    fn control_characters_in_description_sanitized() {
        let json = "{\n  \"schemaVersion\": 1,\n  \"id\": \"testmod\",\n  \"version\": \"1.0.0\",\n  \"description\": \"Line 1\nLine 2\twith tab\"\n}";
        let meta = parse(json).unwrap();
        assert_eq!(
            meta.description.as_deref(),
            Some("Line 1\nLine 2\twith tab")
        );
    }
}
