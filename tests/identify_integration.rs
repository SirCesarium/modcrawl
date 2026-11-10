#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs::File;
use std::process::Command;

use modcrawl::{ForgeModFormat, ModType, PluginType, identify, identify_reader};

#[test]
fn test_identify_fabric_jar() {
    let jar = common::fabric_jar("my_fabric_mod", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify Fabric mod");
    assert_eq!(detected, ModType::Fabric);

    let mut file = File::open(jar.path()).expect("failed to open file");
    let detected_reader = identify_reader(&mut file).expect("failed to identify from reader");
    assert_eq!(detected_reader, ModType::Fabric);
}

#[test]
fn test_identify_quilt_jar() {
    let jar = common::quilt_jar("my_quilt_mod", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify Quilt mod");
    assert_eq!(detected, ModType::Quilt);
}

#[test]
fn test_identify_forge_modern_jar() {
    let jar = common::forge_modern_jar("my_forge_mod", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify Forge modern mod");
    assert_eq!(detected, ModType::Forge(ForgeModFormat::ModsToml));
}

#[test]
fn test_identify_neoforge_jar() {
    let jar = common::neoforge_jar("my_neoforge_mod", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify NeoForge mod");
    assert_eq!(detected, ModType::NeoForge);
}

#[test]
fn test_identify_forge_legacy_jar() {
    let jar = common::forge_legacy_jar("my_legacy_mod", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify Forge legacy mod");
    assert_eq!(detected, ModType::Forge(ForgeModFormat::McmodInfo));
}

#[test]
fn test_identify_bukkit_jar() {
    let jar = common::bukkit_jar("MyBukkitPlugin", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify Bukkit plugin");
    assert_eq!(detected, ModType::Plugin(PluginType::Bukkit));
}

#[test]
fn test_identify_paper_jar() {
    let jar = common::paper_jar("MyPaperPlugin", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify Paper plugin");
    assert_eq!(detected, ModType::Plugin(PluginType::Paper));
}

#[test]
fn test_identify_velocity_jar() {
    let jar = common::velocity_jar("my_velocity_plugin", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify Velocity plugin");
    assert_eq!(detected, ModType::Plugin(PluginType::Velocity));
}

#[test]
fn test_identify_bungee_jar() {
    let jar = common::bungee_jar("MyBungeePlugin", "1.0.0");
    let detected = identify(jar.path()).expect("failed to identify Bungee plugin");
    assert_eq!(detected, ModType::Plugin(PluginType::Bungee));
}

#[test]
fn test_identify_priority_paper_over_bukkit() {
    // A plugin with both paper-plugin.yml and plugin.yml should be detected as Paper
    let jar = common::TempJar::from_entries(&[
        (
            "paper-plugin.yml",
            b"name: DualPlugin\nversion: 1.0\nmain: org.example.Dual\napi-version: '1.20'\n",
        ),
        (
            "plugin.yml",
            b"name: DualPlugin\nversion: 1.0\nmain: org.example.Dual\n",
        ),
    ]);
    let detected = identify(jar.path()).expect("failed to identify dual paper/bukkit");
    assert_eq!(detected, ModType::Plugin(PluginType::Paper));
}

#[test]
fn test_identify_priority_fabric_over_quilt() {
    // A mod shipping both fabric.mod.json and quilt.mod.json is detected as Fabric
    let jar = common::TempJar::from_entries(&[
        (
            "fabric.mod.json",
            br#"{"schemaVersion":1,"id":"dual","version":"1.0"}"#,
        ),
        (
            "quilt.mod.json",
            br#"{"schema_version":1,"quilt_loader":{"group":"org.example","id":"dual","version":"1.0"}}"#,
        ),
    ]);
    let detected = identify(jar.path()).expect("failed to identify dual fabric/quilt");
    assert_eq!(detected, ModType::Fabric);
}

#[test]
fn test_identify_embedded_shell_jar() {
    // Shell jar without outer metadata, but containing embedded jar in META-INF/jarjar/
    let jar = common::embedded_shell_fixture();
    let detected = identify(jar.path()).expect("failed to identify embedded shell");
    assert_eq!(detected, ModType::Fabric);
}

#[test]
fn test_identify_unknown_zip() {
    let jar = common::TempJar::from_entries(&[
        ("assets/icon.png", b"fake image bytes"),
        ("README.txt", b"plain readme"),
    ]);
    let detected = identify(jar.path()).expect("failed to inspect unknown zip");
    assert_eq!(detected, ModType::Unknown);
}

#[test]
fn test_identify_invalid_zip_error() {
    let not_a_zip = common::TempJar::from_bytes(b"not a valid zip file header");
    let err = identify(not_a_zip.path()).expect_err("expected error on invalid zip");
    assert!(
        matches!(err, modcrawl::Error::ZipCrawl(_)),
        "expected ZipCrawl error, got {err:?}"
    );
}

#[test]
fn test_identify_json_single() {
    let jar = common::fabric_jar("json_single", "1.0.0");
    let path = jar.path().to_str().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_modcrawl"))
        .args(["type", "--json", path])
        .output()
        .expect("failed to run modcrawl");
    assert!(output.status.success());
    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output is not valid JSON");
    assert_eq!(v.get("type").and_then(|t| t.as_str()), Some("Fabric"));
    assert!(v.get("file").is_some());
}

#[test]
fn test_identify_json_multi() {
    let jar1 = common::fabric_jar("json_multi_a", "1.0.0");
    let jar2 = common::forge_modern_jar("json_multi_b", "1.0.0");
    let output = Command::new(env!("CARGO_BIN_EXE_modcrawl"))
        .args([
            "type",
            "--json",
            jar1.path().to_str().unwrap(),
            jar2.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to run modcrawl");
    assert!(output.status.success());
    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output is not valid JSON");
    let arr = v
        .as_array()
        .expect("expected JSON array for multi-file output");
    assert_eq!(arr.len(), 2);
    let types: Vec<&str> = arr
        .iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()))
        .collect();
    assert!(types.contains(&"Fabric"));
    assert!(types.contains(&"Forge (mods.toml)"));
}
