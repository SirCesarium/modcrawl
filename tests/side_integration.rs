#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs::File;

use modcrawl::{
    ForgeModFormat, ModType, PluginType, Side, SideReport, read_metadata, side, side_reader,
    side_report, side_report_reader,
};

#[cfg(feature = "classfile")]
use modcrawl::{find_duplicates, find_duplicates_buffers, group_conflicts};

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

/// A Jar detected as `NeoForge` must still honour a `fabric.mod.json` shipped
/// alongside it, since that is the only place the client-only side is stated.
#[test]
fn test_side_dual_loader_uses_fabric_environment() {
    let neoforge = r#"modLoader="javafml"
loaderVersion="[47,)"
license="MIT"

[[mods]]
modId="dual"
version="1.0.0"

[[dependencies.dual]]
modId="neoforge"
mudatory=true
versionRange="[47.1.0,)"
ordering="NONE"
side="BOTH"
"#;
    let fabric = r#"{"schemaVersion":1,"id":"dual","version":"1.0.0","environment":"client"}"#;
    let jar = common::TempJar::from_entries(&[
        ("META-INF/neoforge.mods.toml", neoforge.as_bytes()),
        ("fabric.mod.json", fabric.as_bytes()),
    ]);

    let report = side_report(jar.path()).expect("failed to classify dual-loader jar");
    assert_eq!(report.mod_type, ModType::NeoForge);
    assert_eq!(report.side, Side::Client);
}

/// An explicit `*` in a secondary `fabric.mod.json` lifts an otherwise silent
/// `NeoForge` jar from unknown to both.
#[test]
fn test_side_dual_loader_explicit_both_lifts_unknown() {
    let neoforge = "modLoader=\"javafml\"\nloaderVersion=\"[47,)\"\nlicense=\"MIT\"\n\n[[mods]]\nmodId=\"dual\"\nversion=\"1.0.0\"\n";
    let fabric = r#"{"schemaVersion":1,"id":"dual","version":"1.0.0","environment":"*"}"#;
    let jar = common::TempJar::from_entries(&[
        ("META-INF/neoforge.mods.toml", neoforge.as_bytes()),
        ("fabric.mod.json", fabric.as_bytes()),
    ]);

    let report = side_report(jar.path()).expect("failed to classify dual-loader jar");
    assert_eq!(report.mod_type, ModType::NeoForge);
    assert_eq!(report.side, Side::Both);
}

/// A definitive side from the detected loader is never overridden by the
/// secondary metadata of the other loader.
#[test]
fn test_side_dual_loader_primary_side_wins() {
    let neoforge = "modLoader=\"javafml\"\nloaderVersion=\"[47,)\"\nlicense=\"MIT\"\nclientSideOnly=true\n\n[[mods]]\nmodId=\"dual\"\nversion=\"1.0.0\"\n";
    let fabric = r#"{"schemaVersion":1,"id":"dual","version":"1.0.0","environment":"server"}"#;
    let jar = common::TempJar::from_entries(&[
        ("META-INF/neoforge.mods.toml", neoforge.as_bytes()),
        ("fabric.mod.json", fabric.as_bytes()),
    ]);

    let report = side_report(jar.path()).expect("failed to classify dual-loader jar");
    assert_eq!(report.mod_type, ModType::NeoForge);
    assert_eq!(report.side, Side::Client);
}

/// `both` comes from the loader default, so client-only mixins plus no server
/// payload are enough to narrow the side to `client`.
#[test]
fn test_side_client_only_mixins_narrow_both_to_client() {
    let tomllib = "modLoader=\"javafml\"\nloaderVersion=\"[47,)\"\nlicense=\"MIT\"\n\n[[mods]]\nmodId=\"clientish\"\nversion=\"1.0.0\"\n\n[[dependencies.clientish]]\nmodId=\"forge\"\nmandatory=true\nversionRange=\"[47.1.0,)\"\nordering=\"NONE\"\nside=\"BOTH\"\n";
    let mixins =
        r#"{"required":true,"package":"com.example.mixin","mixins":[],"client":["MixinScreen"]}"#;
    let jar = common::TempJar::from_entries(&[
        ("META-INF/mods.toml", tomllib.as_bytes()),
        ("clientish.mixins.json", mixins.as_bytes()),
        ("com/example/Clientish.class", common::FAKE_CLASS_BYTES),
    ]);

    let report = side_report(jar.path()).expect("failed to classify client-only jar");
    assert_eq!(report.mod_type, ModType::Forge(ForgeModFormat::ModsToml));
    assert_eq!(report.side, Side::Client);
}

/// A shipped `data/` payload (recipes, loot) is server-side content, so the
/// `both` declaration stands.
#[test]
fn test_side_data_payload_keeps_both() {
    let tomllib = "modLoader=\"javafml\"\nloaderVersion=\"[47,)\"\nlicense=\"MIT\"\n\n[[mods]]\nmodId=\"contentful\"\nversion=\"1.0.0\"\n\n[[dependencies.contentful]]\nmodId=\"forge\"\nmandatory=true\nversionRange=\"[47.1.0,)\"\nordering=\"NONE\"\nside=\"BOTH\"\n";
    let mixins =
        r#"{"required":true,"package":"com.example.mixin","mixins":[],"client":["MixinScreen"]}"#;
    let jar = common::TempJar::from_entries(&[
        ("META-INF/mods.toml", tomllib.as_bytes()),
        ("contentful.mixins.json", mixins.as_bytes()),
        ("com/example/Contentful.class", common::FAKE_CLASS_BYTES),
        ("data/contentful/recipe/thing.json", b"{}"),
    ]);

    assert_eq!(side(jar.path()).expect("failed to classify"), Side::Both);
}

/// A class that touches `net/minecraft/server/` proves the mod runs
/// server-side, so it must not be reported as client.
#[cfg(feature = "classfile")]
#[test]
fn test_side_server_class_reference_keeps_both() {
    let tomllib = "modLoader=\"javafml\"\nloaderVersion=\"[47,)\"\nlicense=\"MIT\"\n\n[[mods]]\nmodId=\"serverish\"\nversion=\"1.0.0\"\n\n[[dependencies.serverish]]\nmodId=\"forge\"\nmandatory=true\nversionRange=\"[47.1.0,)\"\nordering=\"NONE\"\nside=\"BOTH\"\n";
    let mixins =
        r#"{"required":true,"package":"com.example.mixin","mixins":[],"client":["MixinScreen"]}"#;
    let server_class: &[u8] =
        b"\xca\xfe\xba\xbe\x00\x00\x00\x34net/minecraft/server/MinecraftServer";
    let jar = common::TempJar::from_entries(&[
        ("META-INF/mods.toml", tomllib.as_bytes()),
        ("serverish.mixins.json", mixins.as_bytes()),
        ("com/example/Serverish.class", server_class),
    ]);

    assert_eq!(side(jar.path()).expect("failed to classify"), Side::Both);
}

/// A `main` entrypoint initialises on both sides, so the `both` declaration of
/// a library stays even when all of its mixins are client-only.
#[test]
fn test_side_main_entrypoint_keeps_both() {
    let manifest = r#"{
      "schemaVersion": 1,
      "id": "bothlib",
      "version": "1.0.0",
      "environment": "*",
      "entrypoints": {"main": ["com.example.BothLib"], "client": ["com.example.BothLibClient"]}
    }"#;
    let mixins =
        r#"{"required":true,"package":"com.example.mixin","mixins":[],"client":["MixinScreen"]}"#;
    let jar = common::TempJar::from_entries(&[
        ("fabric.mod.json", manifest.as_bytes()),
        ("bothlib.mixins.json", mixins.as_bytes()),
        ("com/example/BothLib.class", common::FAKE_CLASS_BYTES),
    ]);

    assert_eq!(side(jar.path()).expect("failed to classify"), Side::Both);
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

/// A Fabric mod whose metadata declares no `environment`, so only the mixin
/// configs can say where it runs.
fn fabric_without_environment(extra: &[(&str, &[u8])]) -> common::TempJar {
    const FABRIC_MOD: &[u8] = br#"{"schemaVersion":1,"id":"side_test","version":"1.0.0"}"#;

    let mut entries = vec![("fabric.mod.json", FABRIC_MOD)];
    entries.extend_from_slice(extra);

    common::TempJar::from_entries(&entries)
}

#[test]
fn test_side_from_root_mixin_config() {
    let jar = fabric_without_environment(&[(
        "side_test.mixins.json",
        br#"{"package":"com.example.mixin","client":["com.example.client.Mixin"]}"#,
    )]);

    let report = side_report(jar.path()).expect("failed to classify");
    assert_eq!(report.mod_type, ModType::Fabric);
    assert_eq!(report.side, Side::Client);

    let jar = fabric_without_environment(&[(
        "side_test.mixins.json",
        br#"{"package":"com.example.mixin","server":["com.example.server.Mixin"]}"#,
    )]);
    assert_eq!(side(jar.path()).expect("failed to classify"), Side::Server);
}

#[test]
fn test_side_from_nested_mixin_config() {
    let inner = common::build_jar(&[
        (
            "fabric.mod.json",
            br#"{"schemaVersion":1,"id":"inner_side","version":"1.0.0"}"#,
        ),
        (
            "inner.mixins.json",
            br#"{"package":"com.example.mixin","client":["com.example.client.Mixin"]}"#,
        ),
    ]);

    let jar = common::TempJar::from_entries(&[
        ("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
        ("META-INF/jarjar/inner-1.0.0.jar", &inner),
    ]);

    let report = side_report(jar.path()).expect("failed to classify shell jar");
    assert_eq!(report.mod_type, ModType::Fabric);
    assert_eq!(report.side, Side::Client);
}

#[test]
fn test_mixin_sections_that_disagree_stay_unknown() {
    let mixed = fabric_without_environment(&[(
        "side_test.mixins.json",
        br#"{"package":"com.example.mixin","client":["a.Client"],"server":["b.Server"]}"#,
    )]);
    assert_eq!(
        side(mixed.path()).expect("failed to classify"),
        Side::Unknown
    );

    // The `mixins` section applies to both sides, so it cannot pick one.
    let common_section = fabric_without_environment(&[(
        "side_test.mixins.json",
        br#"{"package":"com.example.mixin","mixins":["a.Common"],"client":["b.Client"]}"#,
    )]);
    assert_eq!(
        side(common_section.path()).expect("failed to classify"),
        Side::Unknown
    );

    let broken = fabric_without_environment(&[("side_test.mixins.json", b"{ not json")]);
    assert_eq!(
        side(broken.path()).expect("failed to classify"),
        Side::Unknown
    );

    let none = fabric_without_environment(&[]);
    assert_eq!(
        side(none.path()).expect("failed to classify"),
        Side::Unknown
    );
}

#[test]
fn test_metadata_side_wins_over_mixin_configs() {
    let jar = common::TempJar::from_entries(&[
        (
            "fabric.mod.json",
            br#"{"schemaVersion":1,"id":"side_test","version":"1.0.0","environment":"*"}"#,
        ),
        (
            "side_test.mixins.json",
            br#"{"package":"com.example.mixin","server":["com.example.server.Mixin"]}"#,
        ),
    ]);

    assert_eq!(side(jar.path()).expect("failed to classify"), Side::Both);
}

#[test]
fn test_side_report_json_serialization() {
    let report = SideReport {
        mod_type: ModType::Fabric,
        side: Side::Both,
    };
    let json = serde_json::to_string(&report).expect("failed to serialize SideReport");
    assert_eq!(json, r#"{"side":"both"}"#);
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

#[cfg(feature = "classfile")]
#[test]
fn test_group_conflicts_reexported() {
    let b1 = common::build_jar(&[
        ("com/example/A.class", common::FAKE_CLASS_BYTES),
        ("com/example/B.class", common::FAKE_CLASS_BYTES),
    ]);
    let b2 = common::build_jar(&[
        ("com/example/A.class", common::FAKE_CLASS_BYTES),
        ("com/example/B.class", common::FAKE_CLASS_BYTES),
    ]);
    let b3 = common::build_jar(&[("com/example/A.class", common::FAKE_CLASS_BYTES)]);

    let dupes = find_duplicates_buffers(&[("jar1", &b1), ("jar2", &b2), ("jar3", &b3)])
        .expect("failed to find duplicates");

    let conflicts = group_conflicts(&dupes);

    // jar1<->jar2 share A and B; jar1<->jar3 and jar2<->jar3 share only A.
    assert_eq!(conflicts.len(), 3);

    let jar1_jar2 = conflicts
        .iter()
        .find(|e| {
            (e.file_a == "jar1" && e.file_b == "jar2") || (e.file_a == "jar2" && e.file_b == "jar1")
        })
        .expect("expected a jar1<->jar2 conflict entry");
    assert_eq!(jar1_jar2.classes.len(), 2);
}

#[cfg(feature = "classfile")]
#[test]
fn test_mixin_overlaps_reexported() {
    use modcrawl::{
        MixinEntry, MixinOverlap, OverlapEntry, find_mixin_overlaps_buffers, group_mixin_overlaps,
    };

    let entries = vec![
        (
            "mod1.jar".to_string(),
            MixinEntry {
                mixin_class: "net/example/Mix1".to_string(),
                target: "net/minecraft/world/Level".to_string(),
            },
        ),
        (
            "mod2.jar".to_string(),
            MixinEntry {
                mixin_class: "net/example/Mix2".to_string(),
                target: "net/minecraft/world/Level".to_string(),
            },
        ),
    ];

    let overlaps: Vec<MixinOverlap> = group_mixin_overlaps(&entries);
    assert_eq!(overlaps.len(), 1);
    assert_eq!(overlaps[0].target, "net/minecraft/world/Level");
    assert_eq!(overlaps[0].mixins.len(), 2);

    let empty: Vec<OverlapEntry> = vec![];
    assert!(empty.is_empty());

    let from_buffers = find_mixin_overlaps_buffers(&[]).expect("buffer overlap search works");
    assert!(from_buffers.is_empty());
}
