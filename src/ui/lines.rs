//! Shared single-line output: file banner, "nothing found" note, read error.

use std::fmt;

#[cfg(feature = "classfile")]
use crate::core::classfile::SkippedClass;
use anstream::{eprintln, println};
use anstyle::Style;

use super::style::{dim, paint};

/// Width of the rule that separates one JAR's block from the next.
const RULE_WIDTH: usize = 48;

/// Banner above each JAR in a multi-file run.
pub(crate) fn file_header(path: &str) {
    println!();
    println!("{}", paint(dim(), &"─".repeat(RULE_WIDTH)));
    println!("{}", paint(Style::new().bold(), path));
}

/// Note that a search produced nothing. On stderr, so piped stdout stays clean.
pub(crate) fn no_results(msg: &str) {
    let style = Style::new()
        .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow)))
        .dimmed();

    eprintln!("{}", paint(style, msg));
}

/// Report class files that were skipped while scanning a JAR. On stderr, so
/// piped stdout (including `--json`) stays clean.
#[cfg(feature = "classfile")]
pub(crate) fn skipped_classes(path: &str, skipped: &[SkippedClass]) {
    if skipped.is_empty() {
        return;
    }

    let style = Style::new()
        .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow)))
        .dimmed();

    eprintln!(
        "{}",
        paint(
            style,
            &format!(
                "warning: {path}: {} class file(s) could not be parsed",
                skipped.len()
            )
        )
    );

    for class in skipped {
        eprintln!("  {}: {}", paint(dim(), &class.name), class.reason);
    }
}

/// Report a file that could not be read: path in bold, reason in red, on
/// stderr. The path is not repeated when the reason already names it.
pub(crate) fn file_error(path: &str, err: &dyn fmt::Display) {
    let path_style = Style::new().bold();
    let err_style = Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)));
    let message = err.to_string();

    if !path.is_empty() && message.starts_with(path) {
        eprintln!("{}", paint(err_style, &message));
        return;
    }

    eprintln!(
        "{}: {}",
        paint(path_style, path),
        paint(err_style, &message)
    );
}
