#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs::File;

use modcrawl::{
    ForgeModFormat, ModType, PluginType, Side, SideReport, read_metadata, side, side_reader,
    side_report, side_report_reader,
};

#[cfg(feature = "classfile")]
use modcrawl::{find_duplicates, find_duplicates_buffers};

#[test]
fn test_side_fabric_both() {
    let jar = common::fabric_jar("fabric_both", "1.0.0");
    let report = side_report(jar.path()).expect("failed to classify side");
    assert_eq!(report.mod_type, ModType::Fabric);
    assert_eq!(report.side, Side::Both);

    let side_only = side(jar.path()).expect("failed to get side");
    assert_eq!(side_only, Side::Both);

    let mut file = File::open(jar.path()).expect("failed to open jar");
    let reader_report = side_report_reader(&mut file).expect("failed from reader");
    assert_eq!(reader_report, report);

    let mut file = File::open(jar.path()).expect("failed to open jar");
    let reader_side = side_reader(&mut file).expect("failed side from reader");
    assert_eq!(reader_side, Side::Both);
}

#[test]
fn test_side_fabric_client() {
    let manifest = r#"{
      "schemaVersion": 1,
      "id": "fabric_client_only",
      "version": "1.0.0",
      "environment": "client"
    }"#;
    let jar = common::TempJar::from_entries(&[("fabric.mod.json", manifest.as_bytes())]);

    let report = side_report(jar.path()).expect("failed to classify");
    assert_eq!(report.mod_type, ModType::Fabric);
    assert_eq!(report.side, Side::Client);
}

#[test]
fn test_side_bukkit_server() {
    let jar = common::bukkit_jar("BukkitTest", "1.0.0");
    let report = side_report(jar.path()).expect("failed to classify");
    assert_eq!(report.mod_type, ModType::Plugin(PluginType::Bukkit));
    assert_eq!(report.side, Side::Server);
}

#[test]
fn test_side_forge_modern() {
    let jar = common::forge_modern_jar("forge_modern", "1.0.0");
    let report = side_report(jar.path()).expect("failed to classify");
    assert_eq!(report.mod_type, ModType::Forge(ForgeModFormat::ModsToml));
    assert_eq!(report.side, Side::Both);
}

#[test]
fn test_side_unknown_jar() {
    let jar = common::TempJar::from_entries(&[("hello.txt", b"not a mod")]);
    let report = side_report(jar.path()).expect("failed to classify unknown jar");
    assert_eq!(report.mod_type, ModType::Unknown);
    assert_eq!(report.side, Side::Unknown);

    let s = side(jar.path()).expect("failed to get side for unknown jar");
    assert_eq!(s, Side::Unknown);
}

#[test]
fn test_side_report_json_serialization() {
    let report = SideReport {
        mod_type: ModType::Fabric,
        side: Side::Both,
    };
    let json = serde_json::to_string(&report).expect("failed to serialize SideReport");
    assert_eq!(json, r#"{"type":"Fabric","side":"both"}"#);
}

#[test]
fn test_mod_metadata_mod_type() {
    let fabric = common::fabric_jar("meta_test", "1.0.0");
    let meta = read_metadata(fabric.path()).expect("failed to read metadata");
    assert_eq!(meta.mod_type(), ModType::Fabric);

    let bukkit = common::bukkit_jar("BukkitMeta", "1.0.0");
    let meta = read_metadata(bukkit.path()).expect("failed to read metadata");
    assert_eq!(meta.mod_type(), ModType::Plugin(PluginType::Bukkit));
}

#[cfg(feature = "classfile")]
#[test]
fn test_find_duplicates_reexported() {
    let jar1 = common::TempJar::from_entries(&[("com/example/A.class", common::FAKE_CLASS_BYTES)]);
    let jar2 = common::TempJar::from_entries(&[("com/example/A.class", common::FAKE_CLASS_BYTES)]);

    let dupes = find_duplicates(&[jar1.path(), jar2.path()]).expect("failed to find duplicates");
    assert_eq!(dupes.len(), 1);
    assert_eq!(dupes[0].class_name, "com/example/A.class");
    assert_eq!(dupes[0].files.len(), 2);

    let b1 = common::build_jar(&[("com/example/A.class", common::FAKE_CLASS_BYTES)]);
    let b2 = common::build_jar(&[("com/example/A.class", common::FAKE_CLASS_BYTES)]);
    let buf_dupes = find_duplicates_buffers(&[("jar1", &b1), ("jar2", &b2)])
        .expect("failed to find duplicates in buffers");
    assert_eq!(buf_dupes.len(), 1);
}
