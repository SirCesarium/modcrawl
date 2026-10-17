//! The borderless table: the one primitive every listing renders through.
//!
//! No box-drawing, no vertical rules — just a dimmed, letter-spaced header and
//! two-space-gapped columns that line up. That is what makes the whole CLI
//! legible at a glance.

use anstream::println;
use anstyle::Style;

use super::style::{dim, paint};

/// One styled cell. It remembers its *visible* width (the text length without
/// ANSI escapes) so padding still aligns after styling is applied.
pub(crate) struct Cell {
    width: usize,
    rendered: String,
}

/// A plain (unstyled) cell.
pub(crate) fn cell(text: &str) -> Cell {
    styled_cell(Style::new(), text)
}

/// A cell painted with `style`.
pub(crate) fn styled_cell(style: Style, text: &str) -> Cell {
    Cell {
        width: text.chars().count(),
        rendered: paint(style, text),
    }
}

/// Print a borderless table: a dimmed uppercase header row, then one aligned
/// row per entry. Columns are padded to the widest cell they contain.
pub(crate) fn print_table(headers: &[&str], rows: &[Vec<Cell>]) {
    if rows.is_empty() {
        return;
    }

    // Widest visible cell per column, starting from the header widths.
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();

    for row in rows {
        for (idx, c) in row.iter().enumerate() {
            if let Some(w) = widths.get_mut(idx) {
                *w = (*w).max(c.width);
            }
        }
    }

    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let padded = format!("{h:<width$}", width = widths[i]);
            paint(dim(), &padded.to_uppercase())
        })
        .collect();

    println!("{}", header_line.join("  "));

    for row in rows {
        let line: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let pad = " ".repeat(widths[i] - c.width);
                format!("{}{}", c.rendered, pad)
            })
            .collect();

        println!("{}", line.join("  "));
    }
}
