#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::process::Command;

/// Run `modcrawl grep` with `args` and return its stdout.
fn grep(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_modcrawl"))
        .arg("grep")
        .args(args)
        .output()
        .expect("failed to run modcrawl");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout is not utf-8")
}

/// The path of the demo JAR, for as long as `jar` is alive.
fn path_of(jar: &common::TempJar) -> &str {
    jar.path().to_str().unwrap()
}

#[test]
fn matches_are_grouped_under_one_class_header() {
    let jar = common::demo_class_jar();
    let stdout = grep(&["Test", path_of(&jar)]);

    assert_eq!(stdout.matches("com/example/DemoClass.class").count(), 1);
    assert!(stdout.contains("Utf8"));
    assert!(stdout.contains("Class"));
    assert!(!stdout.contains("POOL"), "the table header is gone");
    assert!(!stdout.contains("CLASS"), "class names are not a column");
}

#[test]
fn limit_shows_n_matches_and_reports_the_rest() {
    let jar = common::demo_class_jar();
    let stdout = grep(&["Test", path_of(&jar), "--limit", "1"]);

    // Title, blank line, class header, one match, and the note.
    assert_eq!(stdout.lines().count(), 5);
    assert!(
        stdout.contains("… 1 more match (raise --limit or use --json)"),
        "got {stdout:?}"
    );
}

#[test]
fn limit_zero_prints_only_the_note() {
    let jar = common::demo_class_jar();
    let stdout = grep(&["Test", path_of(&jar), "--limit", "0"]);

    assert_eq!(
        stdout.trim(),
        "… 2 more matches (raise --limit or use --json)"
    );
}

#[test]
fn json_keeps_every_match_despite_limit() {
    let jar = common::demo_class_jar();
    let stdout = grep(&["Test", path_of(&jar), "--json", "--limit", "1"]);

    let matches: serde_json::Value =
        serde_json::from_str(&stdout).expect("output is not valid JSON");
    assert_eq!(matches.as_array().unwrap().len(), 2);
}
