#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs::File;

use modcrawl::{ModMetadata, read_metadata, read_metadata_reader};

#[test]
fn test_metadata_fabric() {
    let jar = common::fabric_jar("test_fabric", "2.1.0");
    let meta = read_metadata(jar.path()).expect("failed to read Fabric metadata");
    match meta {
        ModMetadata::Fabric(m) => {
            assert_eq!(m.id, "test_fabric");
            assert_eq!(m.version, "2.1.0");
            assert_eq!(m.name.as_deref(), Some("Fabric Test Mod"));
            assert_eq!(m.license.as_deref(), Some("MIT"));
        }
        other => panic!("expected ModMetadata::Fabric, got {other:?}"),
    }

    let mut file = File::open(jar.path()).expect("failed to open jar");
    let meta_reader = read_metadata_reader(&mut file).expect("failed to read from reader");
    assert!(matches!(meta_reader, ModMetadata::Fabric(_)));
}

#[test]
fn test_metadata_quilt() {
    let jar = common::quilt_jar("test_quilt", "1.5.0");
    let meta = read_metadata(jar.path()).expect("failed to read Quilt metadata");
    match meta {
        ModMetadata::Quilt(m) => {
            assert_eq!(m.quilt_loader.id, "test_quilt");
            assert_eq!(m.quilt_loader.version.as_deref(), Some("1.5.0"));
            assert_eq!(
                m.quilt_loader.metadata.name.as_deref(),
                Some("Quilt Test Mod")
            );
        }
        other => panic!("expected ModMetadata::Quilt, got {other:?}"),
    }
}

#[test]
fn test_metadata_forge_modern() {
    let jar = common::forge_modern_jar("test_forge", "3.0.0");
    let meta = read_metadata(jar.path()).expect("failed to read Forge modern metadata");
    match meta {
        ModMetadata::ForgeModern(m) => {
            assert_eq!(m.mods.len(), 1);
            assert_eq!(m.mods[0].mod_id, "test_forge");
            assert_eq!(m.mods[0].version.as_deref(), Some("3.0.0"));
        }
        other => panic!("expected ModMetadata::ForgeModern, got {other:?}"),
    }
}

#[test]
fn test_metadata_neoforge() {
    let jar = common::neoforge_jar("test_neoforge", "4.0.0");
    let meta = read_metadata(jar.path()).expect("failed to read NeoForge metadata");
    match meta {
        ModMetadata::NeoForge(m) => {
            assert_eq!(m.mods.len(), 1);
            assert_eq!(m.mods[0].mod_id, "test_neoforge");
            assert_eq!(m.mods[0].version.as_deref(), Some("4.0.0"));
        }
        other => panic!("expected ModMetadata::NeoForge, got {other:?}"),
    }
}

#[test]
fn test_metadata_forge_legacy() {
    let jar = common::forge_legacy_jar("test_legacy", "1.12-1.0");
    let meta = read_metadata(jar.path()).expect("failed to read Forge legacy metadata");
    match meta {
        ModMetadata::ForgeLegacy(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(m[0].modid, "test_legacy");
            assert_eq!(m[0].version.as_deref(), Some("1.12-1.0"));
        }
        other => panic!("expected ModMetadata::ForgeLegacy, got {other:?}"),
    }
}

#[test]
fn test_metadata_bukkit() {
    let jar = common::bukkit_jar("TestBukkit", "1.2.3");
    let meta = read_metadata(jar.path()).expect("failed to read Bukkit metadata");
    match meta {
        ModMetadata::Bukkit(m) => {
            assert_eq!(m.name, "TestBukkit");
            assert_eq!(m.version, "1.2.3");
        }
        other => panic!("expected ModMetadata::Bukkit, got {other:?}"),
    }
}

#[test]
fn test_metadata_paper() {
    let jar = common::paper_jar("TestPaper", "2.3.4");
    let meta = read_metadata(jar.path()).expect("failed to read Paper metadata");
    match meta {
        ModMetadata::Paper(m) => {
            assert_eq!(m.name, "TestPaper");
            assert_eq!(m.version, "2.3.4");
        }
        other => panic!("expected ModMetadata::Paper, got {other:?}"),
    }
}

#[test]
fn test_metadata_velocity() {
    let jar = common::velocity_jar("test_velocity", "0.9.0");
    let meta = read_metadata(jar.path()).expect("failed to read Velocity metadata");
    match meta {
        ModMetadata::Velocity(m) => {
            assert_eq!(m.id, "test_velocity");
            assert_eq!(m.version.as_deref(), Some("0.9.0"));
        }
        other => panic!("expected ModMetadata::Velocity, got {other:?}"),
    }
}

#[test]
fn test_metadata_bungee() {
    let jar = common::bungee_jar("TestBungee", "1.0.1");
    let meta = read_metadata(jar.path()).expect("failed to read Bungee metadata");
    match meta {
        ModMetadata::Bungee(m) => {
            assert_eq!(m.name, "TestBungee");
            assert_eq!(m.version, "1.0.1");
        }
        other => panic!("expected ModMetadata::Bungee, got {other:?}"),
    }
}

#[test]
fn test_metadata_embedded_shell() {
    let jar = common::embedded_shell_fixture();
    let meta = read_metadata(jar.path()).expect("failed to read embedded shell metadata");
    match meta {
        ModMetadata::Fabric(m) => {
            assert_eq!(m.id, "embedded_mod");
            assert_eq!(m.version, "1.2.3");
        }
        other => panic!("expected ModMetadata::Fabric from embedded shell, got {other:?}"),
    }
}

#[test]
fn test_metadata_malformed_json_error() {
    let jar = common::TempJar::from_entries(&[(
        "fabric.mod.json",
        b"{ malformed json string without closing",
    )]);
    let err = read_metadata(jar.path()).expect_err("expected error on malformed json");
    assert!(
        matches!(
            err,
            modcrawl::Error::Json(_) | modcrawl::Error::UnsupportedMetadata(_)
        ),
        "expected Json or UnsupportedMetadata error, got {err:?}"
    );
}

#[test]
fn test_metadata_malformed_toml_error() {
    let jar = common::TempJar::from_entries(&[(
        "META-INF/mods.toml",
        b"modLoader = [unclosed toml array",
    )]);
    let err = read_metadata(jar.path()).expect_err("expected error on malformed toml");
    assert!(
        matches!(
            err,
            modcrawl::Error::Toml(_) | modcrawl::Error::UnsupportedMetadata(_)
        ),
        "expected Toml or UnsupportedMetadata error, got {err:?}"
    );
}

#[test]
fn test_metadata_missing_file_error() {
    let jar = common::TempJar::from_entries(&[("unrelated.txt", b"just a text file")]);
    let err = read_metadata(jar.path()).expect_err("expected error when metadata missing");
    assert!(
        matches!(err, modcrawl::Error::UnsupportedMetadata(_)),
        "expected UnsupportedMetadata error, got {err:?}"
    );
}
