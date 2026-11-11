//! Client/server side classification for mods and plugins.

use std::fmt;
use std::io::{Read, Seek};
use std::path::Path;

use serde::Serialize;
use serde_json::Value;
use zipcrawl::ZipManager;

use crate::core::detect_mod_type::ModType;
use crate::core::embedded::visit_embedded;
use crate::core::identify::identify_impl;
use crate::core::loaders::forge_modern::metadata::{ModsTomlDependency, ModsTomlMetadata};
use crate::core::metadata::{ModMetadata, read_metadata_impl, read_metadata_string};
use crate::error::Result;

/// Where a mod or plugin is meant to run.
///
/// [`Side::Unknown`] is used when the metadata does not state a side. It is
/// different from [`Side::Both`], which is an explicit declaration that the
/// mod runs on both sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Client,
    Server,
    Both,
    Unknown,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Client => "client",
            Self::Server => "server",
            Self::Both => "both",
            Self::Unknown => "unknown",
        };

        f.write_str(label)
    }
}

/// Classification report containing the detected mod/plugin type and its side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SideReport {
    /// Detected mod or plugin type. Kept for Rust callers; only the side is
    /// serialized, because side output carries the side alone.
    #[serde(skip_serializing)]
    pub mod_type: ModType,
    /// Classified side.
    pub side: Side,
}

/// Classify the side and mod type from a JAR file on disk.
///
/// # Errors
///
/// Returns an error if the file cannot be read, is not a valid ZIP archive,
/// or contains malformed metadata.
pub fn side_report(path: &Path) -> Result<SideReport> {
    let mut mng = ZipManager::new(path)?;
    side_report_impl(&mut mng)
}

/// Classify the side and mod type from a ZIP archive read via a `Read + Seek` impl.
///
/// # Errors
///
/// Returns an error if the data cannot be read, is not a valid ZIP archive,
/// or contains malformed metadata.
pub fn side_report_reader<R: Read + Seek>(reader: &mut R) -> Result<SideReport> {
    let mut mng = ZipManager::from_reader(reader)?;
    side_report_impl(&mut mng)
}

/// Classify where a mod or plugin runs from a JAR file on disk.
///
/// # Errors
///
/// Returns an error if the file cannot be read, is not a valid ZIP archive,
/// or contains malformed metadata.
pub fn side(path: &Path) -> Result<Side> {
    side_report(path).map(|r| r.side)
}

/// Classify where a mod or plugin runs from a ZIP archive read via a `Read + Seek` impl.
///
/// # Errors
///
/// Returns an error if the data cannot be read, is not a valid ZIP archive,
/// or contains malformed metadata.
pub fn side_reader<R: Read + Seek>(reader: &mut R) -> Result<Side> {
    side_report_reader(reader).map(|r| r.side)
}

fn side_report_impl<R: Read + Seek>(mng: &mut ZipManager<R>) -> Result<SideReport> {
    let mod_type = identify_impl(mng)?;
    if mod_type == ModType::Unknown {
        return Ok(SideReport {
            mod_type: ModType::Unknown,
            side: Side::Unknown,
        });
    }

    let metadata = read_metadata_impl(mng)?;
    let side = match classify(&metadata) {
        Side::Unknown => mixin_side(mng)?,
        side => side,
    };

    Ok(SideReport { mod_type, side })
}

/// Classify where `metadata` says the mod or plugin runs.
///
/// Fabric and Quilt use their `environment` field. Forge and `NeoForge` use
/// `clientSideOnly`, falling back to the sides declared by their dependencies.
/// Server plugins are always [`Side::Server`]. Legacy Forge metadata does not
/// declare a side, so it is [`Side::Unknown`].
///
/// Metadata alone cannot say more than this: `side` refines [`Side::Unknown`]
/// with the mixin configs of the artifact, which this function cannot see.
#[must_use]
pub fn classify(metadata: &ModMetadata) -> Side {
    match metadata {
        ModMetadata::Fabric(m) => m
            .environment
            .as_deref()
            .map_or(Side::Unknown, environment_side),
        ModMetadata::Quilt(m) => m
            .minecraft
            .as_ref()
            .and_then(|m| m.environment.as_deref())
            .or(m.environment.as_deref())
            .or(m.quilt_loader.environment.as_deref())
            .map_or(Side::Unknown, environment_side),
        ModMetadata::ForgeModern(m) | ModMetadata::NeoForge(m) => forge_side(m),
        ModMetadata::Bukkit(_)
        | ModMetadata::Paper(_)
        | ModMetadata::Bungee(_)
        | ModMetadata::Velocity(_) => Side::Server,
        ModMetadata::ForgeLegacy(_) => Side::Unknown,
    }
}

/// Map a Fabric/Quilt `environment` value to a side.
fn environment_side(value: &str) -> Side {
    match value.trim().to_ascii_lowercase().as_str() {
        "client" => Side::Client,
        "server" => Side::Server,
        "*" | "both" => Side::Both,
        _ => Side::Unknown,
    }
}

/// Classify a Forge or `NeoForge` mod.
fn forge_side(metadata: &ModsTomlMetadata) -> Side {
    match metadata.client_side_only {
        Some(true) => Side::Client,
        Some(false) => Side::Both,
        None => dependency_side(metadata),
    }
}

/// Infer a side from the sides declared by the dependencies.
///
/// The platform dependencies — `minecraft`, `neoforge`, `forge` — decide when
/// they declare one: a mod that needs the game on a single side only runs
/// there, while optional integrations routinely pull in the other side and
/// would otherwise drown that out. Without a platform dependency, a side is
/// reported only when every declared dependency agrees; mixed or unrecognized
/// sides stay [`Side::Unknown`].
fn dependency_side(metadata: &ModsTomlMetadata) -> Side {
    let dependencies: Vec<&ModsTomlDependency> = metadata.dependencies.values().flatten().collect();

    let platform: Vec<Side> = dependencies
        .iter()
        .filter(|dep| is_platform(&dep.mod_id))
        .filter_map(|dep| declared_side(dep))
        .collect();

    if !platform.is_empty() {
        return unanimous(platform);
    }

    unanimous(dependencies.iter().filter_map(|dep| declared_side(dep)))
}

/// Dependencies that describe the platform itself rather than an integration.
const PLATFORM_DEPS: [&str; 3] = ["minecraft", "neoforge", "forge"];

/// Whether `mod_id` names one of the platform dependencies.
fn is_platform(mod_id: &str) -> bool {
    PLATFORM_DEPS
        .iter()
        .any(|id| mod_id.eq_ignore_ascii_case(id))
}

/// Map a declared dependency side, ignoring values that name no side.
fn declared_side(dependency: &ModsTomlDependency) -> Option<Side> {
    let declared = dependency.side.as_deref()?.trim().to_ascii_uppercase();

    match declared.as_str() {
        "CLIENT" => Some(Side::Client),
        "SERVER" => Some(Side::Server),
        "BOTH" => Some(Side::Both),
        _ => None,
    }
}

/// The side every entry agrees on, or [`Side::Unknown`] when they differ or
/// none declares one.
fn unanimous(sides: impl IntoIterator<Item = Side>) -> Side {
    let mut sides = sides.into_iter();

    let Some(first) = sides.next() else {
        return Side::Unknown;
    };

    if sides.all(|side| side == first) {
        first
    } else {
        Side::Unknown
    }
}

/// Side implied by the mixin configs shipped in the artifact.
///
/// This only runs when the metadata declared no side. A `*.mixins.json` sorts
/// its mixins into `client` and `server` sections, while the `mixins` section
/// applies to both sides, so a mod whose mixins all sit in one sided section
/// runs there. Common mixins, mixins on both sides, or no mixin at all stay
/// [`Side::Unknown`]. Configs that cannot be read or parsed are ignored, so a
/// broken one never fails classification.
fn mixin_side<R: Read + Seek>(mng: &mut ZipManager<R>) -> Result<Side> {
    let mut counts = MixinCounts::default();

    for name in mixin_configs(mng)? {
        if let Ok(raw) = read_metadata_string(mng, &name) {
            counts.add(&raw);
        }
    }

    visit_embedded(mng, |_path, nested| {
        for name in mixin_configs(nested)? {
            if let Ok(raw) = read_metadata_string(nested, &name) {
                counts.add(&raw);
            }
        }

        Ok(())
    })?;

    Ok(counts.side())
}

/// Names of the mixin configs at the root of one archive.
fn mixin_configs<R: Read + Seek>(mng: &mut ZipManager<R>) -> Result<Vec<String>> {
    Ok(mng
        .entries()?
        .iter()
        .filter(|entry| !entry.is_dir && is_mixin_config(&entry.name))
        .map(|entry| entry.name.clone())
        .collect())
}

fn is_mixin_config(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".mixins.json")
}

/// Mixin entries counted across every `*.mixins.json` of an archive.
#[derive(Default)]
struct MixinCounts {
    client: usize,
    server: usize,
    common: usize,
}

impl MixinCounts {
    /// Count the `client`, `server`, and `mixins` sections of one config.
    fn add(&mut self, raw: &str) {
        let Ok(config) = serde_json::from_str::<Value>(raw) else {
            return;
        };

        self.client += section_len(&config, "client");
        self.server += section_len(&config, "server");
        self.common += section_len(&config, "mixins");
    }

    /// The sided section every counted mixin belongs to.
    fn side(&self) -> Side {
        match (self.client, self.server, self.common) {
            (client, 0, 0) if client > 0 => Side::Client,
            (0, server, 0) if server > 0 => Side::Server,
            _ => Side::Unknown,
        }
    }
}

/// Number of mixins listed under `key`.
fn section_len(config: &Value, key: &str) -> usize {
    config
        .get(key)
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::loaders::{
        bukkit, bungee, fabric, forge_legacy, forge_modern, paper, quilt, velocity,
    };

    fn fabric(json: &str) -> ModMetadata {
        ModMetadata::Fabric(Box::new(fabric::metadata::parse(json).unwrap()))
    }

    fn quilt(json: &str) -> ModMetadata {
        ModMetadata::Quilt(Box::new(quilt::metadata::parse(json).unwrap()))
    }

    fn forge(toml: &str, kind: fn(Box<ModsTomlMetadata>) -> ModMetadata) -> ModMetadata {
        kind(Box::new(forge_modern::metadata::parse(toml).unwrap()))
    }

    fn modern(toml: &str) -> ModMetadata {
        forge(toml, ModMetadata::ForgeModern)
    }

    fn neoforge(toml: &str) -> ModMetadata {
        forge(toml, ModMetadata::NeoForge)
    }

    fn fabric_with_environment(environment: Option<&str>) -> ModMetadata {
        let environment =
            environment.map_or_else(String::new, |value| format!(r#","environment":"{value}""#));

        fabric(&format!(
            r#"{{"schemaVersion":1,"id":"example","version":"1.0.0"{environment}}}"#
        ))
    }

    #[test]
    fn side_display_is_lowercase() {
        assert_eq!(Side::Client.to_string(), "client");
        assert_eq!(Side::Server.to_string(), "server");
        assert_eq!(Side::Both.to_string(), "both");
        assert_eq!(Side::Unknown.to_string(), "unknown");
    }

    #[test]
    fn fabric_environment() {
        assert_eq!(
            classify(&fabric_with_environment(Some("client"))),
            Side::Client
        );
        assert_eq!(
            classify(&fabric_with_environment(Some("server"))),
            Side::Server
        );
        assert_eq!(classify(&fabric_with_environment(Some("*"))), Side::Both);
        assert_eq!(classify(&fabric_with_environment(Some("both"))), Side::Both);
        assert_eq!(
            classify(&fabric_with_environment(Some("CLIENT"))),
            Side::Client
        );
        assert_eq!(
            classify(&fabric_with_environment(Some("weird"))),
            Side::Unknown
        );
    }

    #[test]
    fn fabric_without_environment_is_unknown() {
        assert_eq!(classify(&fabric_with_environment(None)), Side::Unknown);
    }

    #[test]
    fn quilt_environment() {
        let client = quilt(
            r#"{
                "schema_version": 1,
                "quilt_loader": {"id": "example", "version": "1.0.0"},
                "minecraft": {"environment": "client"}
            }"#,
        );
        assert_eq!(classify(&client), Side::Client);

        let root = quilt(
            r#"{
                "schema_version": 1,
                "quilt_loader": {"id": "example", "version": "1.0.0"},
                "environment": "client"
            }"#,
        );
        assert_eq!(classify(&root), Side::Client);

        let loader = quilt(
            r#"{
                "schema_version": 1,
                "quilt_loader": {"id": "example", "version": "1.0.0", "environment": "server"}
            }"#,
        );
        assert_eq!(classify(&loader), Side::Server);

        let without = quilt(
            r#"{"schema_version": 1, "quilt_loader": {"id": "example", "version": "1.0.0"}}"#,
        );
        assert_eq!(classify(&without), Side::Unknown);
    }

    const MODS: &str = r#"
[[mods]]
modId = "example"
version = "1.0.0"
"#;

    #[test]
    fn forge_client_side_only() {
        let client = modern(&format!("clientSideOnly = true\n{MODS}"));
        assert_eq!(classify(&client), Side::Client);

        let both = modern(&format!("clientSideOnly = false\n{MODS}"));
        assert_eq!(classify(&both), Side::Both);

        let neoforge = neoforge(&format!("clientSideOnly = true\n{MODS}"));
        assert_eq!(classify(&neoforge), Side::Client);
    }

    #[test]
    fn forge_dependency_side_fallback() {
        let client = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "minecraft"
side = "CLIENT"
"#
        ));
        assert_eq!(classify(&client), Side::Client);

        let server = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "minecraft"
side = "SERVER"
"#
        ));
        assert_eq!(classify(&server), Side::Server);

        let both = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "minecraft"
side = "BOTH"
"#
        ));
        assert_eq!(classify(&both), Side::Both);
    }

    #[test]
    fn forge_absent_or_conflicting_side_is_unknown() {
        let absent = modern(MODS);
        assert_eq!(classify(&absent), Side::Unknown);

        let mixed = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "other"
side = "CLIENT"

[[dependencies.example]]
modId = "another"
side = "SERVER"
"#
        ));
        assert_eq!(classify(&mixed), Side::Unknown);
    }

    #[test]
    fn forge_platform_dependency_decides_the_side() {
        // Create's shape: the game runs on both sides while optional
        // integrations are client-only.
        let both = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "neoforge"
side = "BOTH"

[[dependencies.example]]
modId = "minecraft"
side = "BOTH"

[[dependencies.example]]
modId = "ponder"
side = "BOTH"

[[dependencies.example]]
modId = "sodium"
side = "CLIENT"
"#
        ));
        assert_eq!(classify(&both), Side::Both);

        let client = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "minecraft"
side = "CLIENT"

[[dependencies.example]]
modId = "neoforge"
side = "CLIENT"

[[dependencies.example]]
modId = "another"
side = "SERVER"
"#
        ));
        assert_eq!(classify(&client), Side::Client);

        let server = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "minecraft"
side = "SERVER"

[[dependencies.example]]
modId = "another"
side = "BOTH"
"#
        ));
        assert_eq!(classify(&server), Side::Server);
    }

    #[test]
    fn forge_platform_dependencies_that_contradict_are_unknown() {
        let contradictory = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "minecraft"
side = "CLIENT"

[[dependencies.example]]
modId = "forge"
side = "SERVER"
"#
        ));
        assert_eq!(classify(&contradictory), Side::Unknown);

        // A platform dependency without a side carries no information.
        let unsided = modern(&format!(
            r#"{MODS}
[[dependencies.example]]
modId = "minecraft"

[[dependencies.example]]
modId = "other"
side = "CLIENT"

[[dependencies.example]]
modId = "another"
side = "SERVER"
"#
        ));
        assert_eq!(classify(&unsided), Side::Unknown);
    }

    #[test]
    fn plugins_are_server_side() {
        let bukkit = ModMetadata::Bukkit(
            bukkit::metadata::parse("name: Example\nversion: '1.0'\nmain: example.Main\n").unwrap(),
        );
        let paper = ModMetadata::Paper(
            paper::metadata::parse("name: Example\nversion: '1.0'\nmain: example.Main\n").unwrap(),
        );
        let bungee = ModMetadata::Bungee(
            bungee::metadata::parse("name: Example\nversion: '1.0'\nmain: example.Main\n").unwrap(),
        );
        let velocity = ModMetadata::Velocity(
            velocity::metadata::parse(r#"{"id":"example","version":"1.0"}"#).unwrap(),
        );

        for metadata in [bukkit, paper, bungee, velocity] {
            assert_eq!(classify(&metadata), Side::Server);
        }
    }

    #[test]
    fn legacy_forge_is_unknown() {
        let legacy = ModMetadata::ForgeLegacy(forge_legacy::metadata::parse("[]").unwrap());
        assert_eq!(classify(&legacy), Side::Unknown);
    }
}
