use std::collections::BTreeMap;
use std::fmt;
use std::result::Result as StdResult;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use toml::{Value as TomlValue, from_str};

use crate::error::Result;

pub fn parse(input: &str) -> Result<ModsTomlMetadata> {
    Ok(from_str(input)?)
}

/// Read `dependencies` without letting one malformed key sink the whole entry.
///
/// TOML bare keys cannot contain `.`, so `[[dependencies.1.21-1.10]]` nests
/// instead of naming its key: the dependency arrays are still there, just
/// deeper. Collect them from wherever they landed and skip anything that is
/// not made of dependencies.
fn lenient_deps<'de, D>(
    deserializer: D,
) -> StdResult<BTreeMap<String, Vec<ModsTomlDependency>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = TomlValue::deserialize(deserializer)?;
    let Some(table) = raw.as_table() else {
        return Ok(BTreeMap::new());
    };

    Ok(table
        .iter()
        .map(|(key, value)| (key.clone(), deps_in(value)))
        .filter(|(_, deps)| !deps.is_empty())
        .collect())
}

/// Gather every dependency array below `value`, however deeply it is nested.
fn deps_in(value: &TomlValue) -> Vec<ModsTomlDependency> {
    match value {
        TomlValue::Array(items) => items
            .iter()
            .filter_map(|item| ModsTomlDependency::deserialize(item.clone()).ok())
            .collect(),
        TomlValue::Table(table) => table.values().flat_map(deps_in).collect(),
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModsTomlMetadata {
    #[serde(rename = "modLoader")]
    pub mod_loader: Option<String>,
    #[serde(rename = "loaderVersion")]
    pub loader_version: Option<String>,
    #[serde(rename = "issueTrackerURL")]
    pub issue_tracker_url: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub mods: Vec<ModsTomlMod>,
    #[serde(default, deserialize_with = "lenient_deps")]
    pub dependencies: BTreeMap<String, Vec<ModsTomlDependency>>,
    #[serde(rename = "displayURL")]
    pub display_url: Option<String>,
    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,
    pub authors: Option<String>,
    pub credits: Option<String>,
    #[serde(rename = "showAsResourcePack")]
    pub show_as_resource_pack: Option<bool>,
    #[serde(rename = "clientSideOnly")]
    pub client_side_only: Option<bool>,
    #[serde(default)]
    pub mixins: Vec<ModsTomlMixin>,
    #[serde(default)]
    pub modproperties: BTreeMap<String, Value>,
    #[serde(rename = "enumExtensions")]
    pub enum_extensions: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModsTomlMod {
    #[serde(rename = "modId")]
    pub mod_id: String,
    pub version: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "displayURL")]
    pub display_url: Option<String>,
    pub authors: Option<Value>,
    pub description: Option<String>,
    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,
    pub license: Option<String>,
    pub credits: Option<String>,
    #[serde(rename = "updateJSONURL")]
    pub update_jsonurl: Option<String>,
    #[serde(rename = "displayTest")]
    pub display_test: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModsTomlDependency {
    #[serde(rename = "modId")]
    pub mod_id: String,
    #[serde(rename = "type")]
    pub dep_type: Option<String>,
    pub mandatory: Option<bool>,
    #[serde(rename = "versionRange")]
    pub version_range: Option<String>,
    pub ordering: Option<String>,
    pub side: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModsTomlMixin {
    pub config: String,
}

impl fmt::Display for ModsTomlMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(l) = &self.mod_loader {
            writeln!(f, "Loader:   {l}")?;
        }
        if let Some(lv) = &self.loader_version {
            writeln!(f, "LoaderV:  {lv}")?;
        }
        for mod_entry in &self.mods {
            writeln!(f, "Mod ID:   {}", mod_entry.mod_id)?;
            if let Some(n) = &mod_entry.display_name {
                writeln!(f, "Name:     {n}")?;
            }
            if let Some(d) = &mod_entry.description {
                writeln!(f, "About:    {d}")?;
            }
            if let Some(v) = &mod_entry.version {
                writeln!(f, "Version:  {v}")?;
            }
            if let Some(a) = &mod_entry.authors {
                writeln!(f, "Authors:  {a}")?;
            }
        }
        if !self.dependencies.is_empty() {
            use std::collections::{BTreeMap, HashSet};
            writeln!(f, "Dependencies:")?;
            let mut merged: BTreeMap<&str, (&str, HashSet<Option<&str>>)> = BTreeMap::new();
            for deps in self.dependencies.values() {
                for dep in deps {
                    let (vr, sides) = merged.entry(dep.mod_id.as_str()).or_insert_with(|| {
                        (dep.version_range.as_deref().unwrap_or("*"), HashSet::new())
                    });
                    sides.insert(dep.side.as_deref());
                    let new_vr = dep.version_range.as_deref().unwrap_or("*");
                    if new_vr != *vr {
                        *vr = new_vr;
                    }
                }
            }
            for (mod_id, (vr, sides)) in &merged {
                let suffix = match sides.len() {
                    1 => match sides.iter().next().and_then(|s| *s) {
                        Some("CLIENT") => " (Client)",
                        Some("SERVER") => " (Server)",
                        Some("BOTH") => " (Both)",
                        _ => "",
                    },
                    _ => "",
                };
                writeln!(f, "  - {mod_id} ({vr}){suffix}")?;
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
        let toml = r#"
[[mods]]
modId = "testmod"
version = "1.0.0"
"#;
        let meta = parse(toml).unwrap();
        assert_eq!(meta.mods.len(), 1);
        assert_eq!(meta.mods[0].mod_id, "testmod");
    }

    #[test]
    fn full() {
        let toml = r#"
modLoader = "javafml"
loaderVersion = "[60,)"
issueTrackerURL = "https://example.com/issues"
license = "MIT"

[[mods]]
modId = "testmod"
version = "1.0.0"
displayName = "Test Mod"
description = "A test mod"
authors = "Alice"

[[dependencies.testmod]]
modId = "minecraft"
type = "required"
versionRange = ">=1.20"
mandatory = true
"#;
        let meta = parse(toml).unwrap();
        assert_eq!(meta.mod_loader.as_deref(), Some("javafml"));
        assert_eq!(meta.loader_version.as_deref(), Some("[60,)"));
        assert_eq!(meta.license.as_deref(), Some("MIT"));
        assert_eq!(meta.mods.len(), 1);
        assert_eq!(meta.mods[0].display_name.as_deref(), Some("Test Mod"));

        let ids: Vec<&str> = meta
            .dependencies
            .values()
            .flatten()
            .map(|dep| dep.mod_id.as_str())
            .collect();
        assert_eq!(ids, ["minecraft"]);
    }

    #[test]
    fn a_dotted_dependency_key_still_yields_the_mod_and_its_deps() {
        // A bare TOML key cannot hold `.`, so this nests instead of naming a key.
        let toml = r#"
modLoader = "javafml"
license = "GLP3"

[[mods]]
modId = "modelfix"
version = "1.21-1.10"
displayName = "Model Gap Fix"

[[dependencies.1.21-1.10]]
modId = "neoforge"
mandatory = true
versionRange = "[4,)"
side = "CLIENT"

[[dependencies.1.21-1.10]]
modId = "minecraft"
mandatory = true
versionRange = "[1.21,1.22)"
side = "CLIENT"
"#;
        let meta = parse(toml).unwrap();

        assert_eq!(meta.mods.len(), 1);
        assert_eq!(meta.mods[0].mod_id, "modelfix");
        assert_eq!(meta.mods[0].display_name.as_deref(), Some("Model Gap Fix"));

        let ids: Vec<&str> = meta
            .dependencies
            .values()
            .flatten()
            .map(|dep| dep.mod_id.as_str())
            .collect();
        assert_eq!(ids, ["neoforge", "minecraft"]);
    }

    #[test]
    fn part_of_a_dependency_section_that_is_not_a_dependency_is_skipped() {
        let toml = r#"
[[mods]]
modId = "testmod"

[[dependencies.testmod]]
modId = "minecraft"

[[dependencies.testmod]]
mandatory = true
versionRange = "[1.21,1.22)"
"#;
        let meta = parse(toml).unwrap();
        let ids: Vec<&str> = meta
            .dependencies
            .values()
            .flatten()
            .map(|dep| dep.mod_id.as_str())
            .collect();
        assert_eq!(ids, ["minecraft"]);
    }

    #[test]
    fn display_output() {
        let toml = r#"
[[mods]]
modId = "testmod"
version = "1.0.0"
displayName = "Test Mod"
"#;
        let meta = parse(toml).unwrap();
        let out = meta.to_string();
        assert!(out.contains("Mod ID:   testmod"));
        assert!(out.contains("Name:     Test Mod"));
        assert!(out.contains("Version:  1.0.0"));
    }
}
