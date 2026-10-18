//! Small single-line output helpers shared across the report commands: the
//! file banner for multi-file runs, the "nothing found" note, and the per-file
//! read error. Keeping them together is what makes every subcommand announce
//! empty and failed files the same way.

use std::fmt;

use anstream::{eprintln, println};
use anstyle::Style;

use super::style::{dim, paint};

/// Width of the rule that separates one JAR's block from the next.
const RULE_WIDTH: usize = 48;

/// The dimmed file banner printed above each JAR in a multi-file run, so the
/// reader always knows which file the following lines describe. A faint rule
/// above the path keeps adjacent JARs from running together.
pub(crate) fn file_header(path: &str) {
    println!();
    println!("{}", paint(dim(), &"─".repeat(RULE_WIDTH)));
    println!("{}", paint(Style::new().bold(), path));
}

/// A note that a search produced nothing. Goes to stderr so it never pollutes
/// piped stdout, and is styled so it stands out from real results.
pub(crate) fn no_results(msg: &str) {
    let style = Style::new()
        .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow)))
        .dimmed();

    eprintln!("{}", paint(style, msg));
}

/// Report that a file could not be read. Goes to stderr, with the path in
/// bold and the reason in the standard error color.
pub(crate) fn file_error(path: &str, err: &dyn fmt::Display) {
    let path_style = Style::new().bold();
    let err_style = Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)));

    eprintln!(
        "{}: {}",
        paint(path_style, path),
        paint(err_style, &err.to_string())
    );
}
