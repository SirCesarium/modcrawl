//! Shared single-line output: file banner, "nothing found" note, read error.

use std::fmt;

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

/// Report a file that could not be read: path in bold, reason in red, on
/// stderr.
pub(crate) fn file_error(path: &str, err: &dyn fmt::Display) {
    let path_style = Style::new().bold();
    let err_style = Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)));

    eprintln!(
        "{}: {}",
        paint(path_style, path),
        paint(err_style, &err.to_string())
    );
}
