//! `modcrawl grep` — constant-pool matches grouped by class file.

use anstream::println;
use anstyle::Style;

use super::lines::no_results;
use super::style::{dim, paint};
use super::text;
use crate::core::classfile::GrepMatch;

/// Indent of a match line under its class file.
const INDENT: usize = 2;
/// Columns between the pool tag and the value.
const GAP: usize = 2;

/// Render the constant-pool entries matching the needle: one block per class
/// file, then one line per match, continued on the next line under the value
/// when it does not fit the terminal.
pub(crate) fn render(matches: &[GrepMatch]) {
    if matches.is_empty() {
        no_results("No matches found.");
        return;
    }

    println!("{}", paint(Style::new().bold(), "Constant pool matches"));
    println!();

    let width = text::terminal_width();
    let tag_width = matches
        .iter()
        .map(|m| text::width(&m.pool_tag))
        .max()
        .unwrap_or(0);
    let value_column = INDENT + tag_width + GAP;
    let indent = " ".repeat(INDENT);
    let continuation = " ".repeat(value_column);
    let mut class: Option<&str> = None;

    for m in matches {
        if class != Some(m.class_file.as_str()) {
            class = Some(&m.class_file);
            print_class(width, &m.class_file);
        }

        let value = width.map_or_else(
            || vec![m.value.clone()],
            |columns| text::wrap(&m.value, columns.saturating_sub(value_column).max(1)),
        );

        for (index, line) in value.iter().enumerate() {
            if index == 0 {
                println!(
                    "{indent}{}  {line}",
                    paint(dim(), &text::pad(&m.pool_tag, tag_width))
                );
            } else {
                println!("{continuation}{line}");
            }
        }
    }
}

/// Note that a listing stopped before the last match.
pub(crate) fn more_matches(count: usize) {
    let noun = if count == 1 { "match" } else { "matches" };

    println!(
        "{}",
        paint(
            dim(),
            &format!("… {count} more {noun} (raise --limit or use --json)")
        )
    );
}

/// The class file a block of matches belongs to, kept on one line per terminal
/// line so a deep package path never wraps back to the margin.
fn print_class(width: Option<usize>, class_file: &str) {
    for line in width.map_or_else(
        || vec![class_file.to_owned()],
        |columns| text::wrap(class_file, columns),
    ) {
        println!("{}", paint(Style::new().bold(), &line));
    }
}
