//! Client/server side classification for mods and plugins.

use std::fmt;

use serde::Serialize;

use crate::core::loaders::forge_modern::metadata::ModsTomlMetadata;
use crate::core::metadata::ModMetadata;

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

/// Classify where `metadata` says the mod or plugin runs.
///
/// Fabric and Quilt use their `environment` field. Forge and `NeoForge` use
/// `clientSideOnly`, falling back to the sides declared by their dependencies.
/// Server plugins are always [`Side::Server`]. Legacy Forge metadata does not
/// declare a side, so it is [`Side::Unknown`].
#[must_use]
pub fn classify(metadata: &ModMetadata) -> Side {
    match metadata {
        ModMetadata::Fabric(m) => m.environment.as_deref().map_or(Side::Unknown, environment_side),
        ModMetadata::Quilt(m) => m
            .minecraft
            .as_ref()
            .and_then(|m| m.environment.as_deref())
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

/// Conservative fallback: infer a side only when every declared dependency side
/// agrees. Mixed or unrecognized sides stay unknown.
fn dependency_side(metadata: &ModsTomlMetadata) -> Side {
    let mut client = false;
    let mut server = false;
    let mut both = false;

    for side in metadata
        .dependencies
        .values()
        .flatten()
        .filter_map(|dep| dep.side.as_deref())
    {
        match side.trim().to_ascii_uppercase().as_str() {
            "CLIENT" => client = true,
            "SERVER" => server = true,
            "BOTH" => both = true,
            _ => {}
        }
    }

    match (client, server, both) {
        (true, false, false) => Side::Client,
        (false, true, false) => Side::Server,
        (false, false, true) => Side::Both,
        _ => Side::Unknown,
    }
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

    fn fabric_environment(environment: Option<&str>) -> ModMetadata {
        let environment = environment.map_or_else(String::new, |value| {
            format!(r#","environment":"{value}""#)
        });

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
            classify(&fabric_environment(Some("client"))),
            Side::Client
        );
        assert_eq!(
            classify(&fabric_environment(Some("server"))),
            Side::Server
        );
        assert_eq!(classify(&fabric_environment(Some("*"))), Side::Both);
        assert_eq!(classify(&fabric_environment(Some("both"))), Side::Both);
        assert_eq!(classify(&fabric_environment(Some("CLIENT"))), Side::Client);
        assert_eq!(classify(&fabric_environment(Some("weird"))), Side::Unknown);
    }

    #[test]
    fn fabric_without_environment_is_unknown() {
        assert_eq!(classify(&fabric_environment(None)), Side::Unknown);
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
modId = "minecraft"
side = "CLIENT"

[[dependencies.example]]
modId = "other"
side = "SERVER"
"#
        ));
        assert_eq!(classify(&mixed), Side::Unknown);
    }

    #[test]
    fn plugins_are_server_side() {
        let bukkit = ModMetadata::Bukkit(
            bukkit::metadata::parse("name: Example\nversion: '1.0'\nmain: example.Main\n")
                .unwrap(),
        );
        let paper = ModMetadata::Paper(
            paper::metadata::parse("name: Example\nversion: '1.0'\nmain: example.Main\n")
                .unwrap(),
        );
        let bungee = ModMetadata::Bungee(
            bungee::metadata::parse("name: Example\nversion: '1.0'\nmain: example.Main\n")
                .unwrap(),
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
