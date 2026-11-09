#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::env;
use std::path::PathBuf;

use modcrawl::{ForgeModFormat, ModType, PluginType, analyze, classify, identify, read_metadata};

#[cfg(feature = "classfile")]
use modcrawl::{list_classes, mixins};

fn fixtures_dir() -> PathBuf {
    env::var("SMOKE_FIXTURES_DIR").map_or_else(|_| PathBuf::from(".smoke_fixtures"), PathBuf::from)
}

fn fixture_path(name: &str) -> Option<PathBuf> {
    let p = fixtures_dir().join(name);
    if p.exists() { Some(p) } else { None }
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_appleskin_fabric() {
    let path = fixture_path("appleskin-fabric-mc1.20.1-2.5.2.jar")
        .expect("appleskin-fabric fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify AppleSkin Fabric");
    assert_eq!(detected, ModType::Fabric);

    let meta = read_metadata(&path).expect("failed to read AppleSkin metadata");
    match &meta {
        modcrawl::ModMetadata::Fabric(m) => {
            assert_eq!(m.id, "appleskin");
            assert!(m.version.contains("2.5.2"));
        }
        other => panic!("expected Fabric metadata, got {other:?}"),
    }

    let report = analyze(&path, true).expect("failed to analyze AppleSkin deps");
    let dep_names: Vec<&str> = report
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(
        dep_names.contains(&"fabricloader"),
        "missing fabricloader dep"
    );
    assert!(dep_names.contains(&"fabric-api"), "missing fabric-api dep");
    assert!(
        !report.jar_in_jar.is_empty(),
        "expected jar-in-jar in AppleSkin"
    );

    let side = classify(&meta);
    assert_eq!(side, modcrawl::Side::Both);

    #[cfg(feature = "classfile")]
    {
        let classes = list_classes(&path).expect("failed to list classes in AppleSkin");
        assert!(!classes.is_empty(), "expected classes in AppleSkin");

        let mx = mixins(&path).expect("failed to list mixins in AppleSkin");
        assert!(!mx.is_empty(), "expected mixins in AppleSkin");
    }
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_quilt_loading_screen() {
    let path = fixture_path("quilt_loading_screen-6.1.0+1.20.1.jar")
        .expect("quilt_loading_screen fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify Quilt Loading Screen");
    assert_eq!(detected, ModType::Quilt);

    let meta = read_metadata(&path).expect("failed to read Quilt metadata");
    match meta {
        modcrawl::ModMetadata::Quilt(m) => {
            assert_eq!(m.quilt_loader.id, "quilt_loading_screen");
        }
        other => panic!("expected Quilt metadata, got {other:?}"),
    }

    let report = analyze(&path, false).expect("failed to analyze Quilt deps");
    assert!(
        !report.dependencies.is_empty(),
        "expected dependencies in Quilt mod"
    );
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_ferritecore_neoforge() {
    let path = fixture_path("ferritecore-9.0.0-neoforge.jar")
        .expect("ferritecore-neoforge fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify FerriteCore NeoForge");
    assert_eq!(detected, ModType::NeoForge);

    let meta = read_metadata(&path).expect("failed to read FerriteCore NeoForge metadata");
    match meta {
        modcrawl::ModMetadata::NeoForge(m) => {
            assert_eq!(m.mods[0].mod_id, "ferritecore");
        }
        other => panic!("expected NeoForge metadata, got {other:?}"),
    }

    let report = analyze(&path, false).expect("failed to analyze FerriteCore NeoForge deps");
    let dep_names: Vec<&str> = report
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(
        dep_names.contains(&"neoforge"),
        "missing neoforge dependency"
    );
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_ferritecore_forge_modern() {
    let path = fixture_path("ferritecore-6.0.1-forge.jar")
        .expect("ferritecore-forge fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify FerriteCore Forge Modern");
    assert_eq!(detected, ModType::Forge(ForgeModFormat::ModsToml));

    let meta = read_metadata(&path).expect("failed to read FerriteCore Forge Modern metadata");
    match meta {
        modcrawl::ModMetadata::ForgeModern(m) => {
            assert_eq!(m.mods[0].mod_id, "ferritecore");
        }
        other => panic!("expected ForgeModern metadata, got {other:?}"),
    }
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_appleskin_forge_legacy() {
    let path = fixture_path("AppleSkin-mc1.12-1.0.14.jar")
        .expect("AppleSkin-1.12 fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify AppleSkin Legacy");
    assert_eq!(detected, ModType::Forge(ForgeModFormat::McmodInfo));

    let meta = read_metadata(&path).expect("failed to read AppleSkin Legacy metadata");
    match meta {
        modcrawl::ModMetadata::ForgeLegacy(m) => {
            assert_eq!(m[0].modid, "appleskin");
        }
        other => panic!("expected ForgeLegacy metadata, got {other:?}"),
    }
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_chunky_bukkit() {
    let path = fixture_path("Chunky-Bukkit-1.5.66.jar")
        .expect("Chunky-Bukkit fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify Chunky Bukkit");
    assert_eq!(detected, ModType::Plugin(PluginType::Bukkit));

    let meta = read_metadata(&path).expect("failed to read Chunky metadata");
    match meta {
        modcrawl::ModMetadata::Bukkit(m) => {
            assert_eq!(m.name, "Chunky");
        }
        other => panic!("expected Bukkit metadata, got {other:?}"),
    }
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_luckperms_velocity() {
    let path = fixture_path("LuckPerms-Velocity-5.5.71.jar")
        .expect("LuckPerms-Velocity fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify LuckPerms Velocity");
    assert_eq!(detected, ModType::Plugin(PluginType::Velocity));

    let meta = read_metadata(&path).expect("failed to read LuckPerms Velocity metadata");
    match meta {
        modcrawl::ModMetadata::Velocity(m) => {
            assert_eq!(m.id, "luckperms");
        }
        other => panic!("expected Velocity metadata, got {other:?}"),
    }
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_bungeeviaproxy_bungee() {
    let path = fixture_path("bungeeviaproxy-1.0-SNAPSHOT.jar")
        .expect("bungeeviaproxy fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify BungeeViaProxy");
    assert_eq!(detected, ModType::Plugin(PluginType::Bungee));

    let meta = read_metadata(&path).expect("failed to read BungeeViaProxy metadata");
    match meta {
        modcrawl::ModMetadata::Bungee(m) => {
            assert_eq!(m.name, "BungeeViaProxy");
        }
        other => panic!("expected Bungee metadata, got {other:?}"),
    }
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_sable_if_present() {
    let Some(path) = fixture_path("sable-fabric-1.21.1-2.0.6.jar") else {
        // Optional heavy fixture
        return;
    };

    let detected = identify(&path).expect("failed to identify Sable");
    assert_eq!(detected, ModType::Fabric);

    let meta = read_metadata(&path).expect("failed to read Sable metadata");
    match meta {
        modcrawl::ModMetadata::Fabric(m) => {
            assert_eq!(m.id, "sable");
        }
        other => panic!("expected Fabric metadata for Sable, got {other:?}"),
    }

    let report = analyze(&path, false).expect("failed to analyze Sable deps");
    let dep_names: Vec<&str> = report
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(dep_names.contains(&"minecraft"));
    assert!(dep_names.contains(&"veil"));

    #[cfg(feature = "classfile")]
    {
        let mx = mixins(&path).expect("failed to list mixins in Sable");
        assert!(!mx.is_empty(), "expected mixins in Sable");
    }
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_jei() {
    let path = fixture_path("jei-1.20.1-forge-15.74.0.234.jar")
        .expect("jei fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify JEI");
    assert_eq!(detected, ModType::Forge(ForgeModFormat::ModsToml));

    let meta = read_metadata(&path).expect("failed to read JEI metadata");
    match meta {
        modcrawl::ModMetadata::ForgeModern(m) => {
            assert_eq!(m.mods[0].mod_id, "jei");
            assert_eq!(m.mods[0].version.as_deref(), Some("15.74.0.234"));
        }
        other => panic!("expected ForgeModern metadata for JEI, got {other:?}"),
    }

    let report = analyze(&path, false).expect("failed to analyze JEI deps");
    let dep_names: Vec<&str> = report
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(dep_names.contains(&"forge"));
    assert!(dep_names.contains(&"minecraft"));
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_sodium() {
    let path = fixture_path("sodium-fabric-0.5.13+mc1.20.1.jar")
        .expect("sodium fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify Sodium");
    assert_eq!(detected, ModType::Fabric);

    let meta = read_metadata(&path).expect("failed to read Sodium metadata");
    match &meta {
        modcrawl::ModMetadata::Fabric(m) => {
            assert_eq!(m.id, "sodium");
            assert!(m.version.contains("0.5.13"));
        }
        other => panic!("expected Fabric metadata for Sodium, got {other:?}"),
    }

    let side = classify(&meta);
    assert_eq!(side, modcrawl::Side::Client);

    let report = analyze(&path, false).expect("failed to analyze Sodium deps");
    let dep_names: Vec<&str> = report
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(dep_names.contains(&"fabricloader"));
    assert!(dep_names.contains(&"minecraft"));
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_architectury() {
    let path = fixture_path("architectury-9.2.14-fabric.jar")
        .expect("architectury fixture not found; run scripts/fetch_modrinth_fixtures.sh");

    let detected = identify(&path).expect("failed to identify Architectury");
    assert_eq!(detected, ModType::Fabric);

    let meta = read_metadata(&path).expect("failed to read Architectury metadata");
    match meta {
        modcrawl::ModMetadata::Fabric(m) => {
            assert_eq!(m.id, "architectury");
            assert_eq!(m.version, "9.2.14");
        }
        other => panic!("expected Fabric metadata for Architectury, got {other:?}"),
    }

    let report = analyze(&path, false).expect("failed to analyze Architectury deps");
    let dep_names: Vec<&str> = report
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(dep_names.contains(&"fabricloader"));
    assert!(dep_names.contains(&"fabric-api"));
    assert!(dep_names.contains(&"minecraft"));
}

#[test]
#[ignore = "requires real downloaded JARs from Modrinth in .smoke_fixtures"]
fn smoke_test_create_if_present() {
    let Some(path) = fixture_path("create-1.20.1-6.0.8.jar") else {
        // Optional heavy fixture
        return;
    };

    let detected = identify(&path).expect("failed to identify Create");
    assert_eq!(detected, ModType::Forge(ForgeModFormat::ModsToml));

    let meta = read_metadata(&path).expect("failed to read Create metadata");
    match meta {
        modcrawl::ModMetadata::ForgeModern(m) => {
            assert_eq!(m.mods[0].mod_id, "create");
            assert_eq!(m.mods[0].version.as_deref(), Some("6.0.8"));
        }
        other => panic!("expected ForgeModern metadata for Create, got {other:?}"),
    }

    let report = analyze(&path, false).expect("failed to analyze Create deps");
    let dep_names: Vec<&str> = report
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(dep_names.contains(&"forge"));
    assert!(dep_names.contains(&"minecraft"));
    assert!(dep_names.contains(&"flywheel"));
    assert!(dep_names.contains(&"ponder"));
    assert!(dep_names.contains(&"jei"));

    #[cfg(feature = "classfile")]
    {
        let mx = mixins(&path).expect("failed to list mixins in Create");
        assert!(!mx.is_empty(), "expected mixins in Create");
    }
}
