#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::BTreeMap;
use std::fs::File;

use modcrawl::{DepKind, analyze, analyze_reader};

#[test]
fn test_analyze_fabric_dependencies() {
    let jar = common::fabric_jar("fabric_dep_test", "1.0.0");
    let report = analyze(jar.path(), false).expect("failed to analyze Fabric dependencies");

    let dep_names: Vec<&str> = report
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(
        dep_names.contains(&"fabricloader"),
        "missing fabricloader dep"
    );
    assert!(dep_names.contains(&"minecraft"), "missing minecraft dep");

    let mut file = File::open(jar.path()).expect("failed to open jar");
    let report_reader = analyze_reader(&mut file, false).expect("failed to analyze from reader");
    assert_eq!(report_reader.dependencies.len(), report.dependencies.len());
}

#[test]
fn test_analyze_quilt_dependencies() {
    let jar = common::quilt_jar("quilt_dep_test", "1.0.0");
    let report = analyze(jar.path(), false).expect("failed to analyze Quilt dependencies");

    assert_eq!(report.dependencies.len(), 1);
    assert_eq!(report.dependencies[0].name, "quilt_loader");
    assert_eq!(report.dependencies[0].kind, DepKind::Required);
}

#[test]
fn test_analyze_forge_modern_dependencies() {
    let jar = common::forge_modern_jar("forge_dep_test", "1.0.0");
    let report = analyze(jar.path(), false).expect("failed to analyze Forge dependencies");

    assert_eq!(report.dependencies.len(), 1);
    assert_eq!(report.dependencies[0].name, "forge");
    assert_eq!(report.dependencies[0].kind, DepKind::Required);
}

#[test]
fn test_analyze_neoforge_dependencies() {
    let jar = common::neoforge_jar("neoforge_dep_test", "1.0.0");
    let report = analyze(jar.path(), false).expect("failed to analyze NeoForge dependencies");

    assert_eq!(report.dependencies.len(), 1);
    assert_eq!(report.dependencies[0].name, "neoforge");
    assert_eq!(report.dependencies[0].kind, DepKind::Required);
}

#[test]
fn test_analyze_bukkit_dependencies() {
    let jar = common::bukkit_jar("BukkitDepTest", "1.0.0");
    let report = analyze(jar.path(), false).expect("failed to analyze Bukkit dependencies");

    let dep_map: BTreeMap<_, _> = report
        .dependencies
        .iter()
        .map(|d| (d.name.as_str(), d.kind))
        .collect();

    assert_eq!(dep_map.get("WorldEdit"), Some(&DepKind::Required));
    assert_eq!(dep_map.get("Vault"), Some(&DepKind::Optional));
}

#[test]
fn test_analyze_paper_dependencies() {
    let jar = common::paper_jar("PaperDepTest", "1.0.0");
    let report = analyze(jar.path(), false).expect("failed to analyze Paper dependencies");

    // Paper handler currently does not extract dependencies from paper-plugin.yml
    assert_eq!(report.dependencies.len(), 0);
}

#[test]
fn test_analyze_jar_in_jar_filtering_and_discovery() {
    let jar = common::jar_in_jar_fixture();

    // With include_jij = false:
    // inner_mod should be filtered out from dependencies because it is bundled!
    // external_mod should remain in dependencies.
    let report_no_jij = analyze(jar.path(), false).expect("failed to analyze without jij");
    assert_eq!(report_no_jij.jar_in_jar.len(), 0);
    assert_eq!(report_no_jij.dependencies.len(), 1);
    assert_eq!(report_no_jij.dependencies[0].name, "external_mod");

    // With include_jij = true:
    // jar_in_jar list contains the inner jar path.
    let report_with_jij = analyze(jar.path(), true).expect("failed to analyze with jij");
    assert_eq!(report_with_jij.jar_in_jar.len(), 1);
    assert_eq!(
        report_with_jij.jar_in_jar[0].path,
        "META-INF/jars/inner_mod-0.5.0.jar"
    );
    assert_eq!(report_with_jij.dependencies.len(), 1);
    assert_eq!(report_with_jij.dependencies[0].name, "external_mod");
}

#[test]
fn test_analyze_embedded_shell() {
    let jar = common::embedded_shell_fixture();
    let report = analyze(jar.path(), false).expect("failed to analyze embedded shell");

    assert_eq!(report.dependencies.len(), 1);
    assert_eq!(report.dependencies[0].name, "minecraft");
}

#[test]
fn test_analyze_invalid_zip_error() {
    let not_a_zip = common::TempJar::from_bytes(b"corrupted header");
    let err = analyze(not_a_zip.path(), false).expect_err("expected error on invalid zip");
    assert!(
        matches!(err, modcrawl::Error::ZipCrawl(_)),
        "expected ZipCrawl error, got {err:?}"
    );
}
