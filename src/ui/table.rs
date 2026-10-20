//! The borderless table primitive every listing renders through.

use anstream::println;
use anstyle::Style;

use super::style::{dim, paint};

/// One styled cell, remembering its visible width so padding still aligns.
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
        width: text
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0),
        rendered: paint(style, text),
    }
}

/// Print a borderless table: dimmed uppercase header, a rule, then one row per
/// entry.
pub(crate) fn print_table(headers: &[&str], rows: &[Vec<Cell>]) {
    if rows.is_empty() {
        return;
    }

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
    print_rule(headers, &widths);

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

/// Dashed rule under the header, stopping where the header text stops: the
/// padding after the last column is not counted. Dashed so it can't be
/// confused with the solid rule `file_header` prints between JARs.
fn print_rule(headers: &[&str], widths: &[usize]) {
    let columns = widths
        .iter()
        .take(widths.len().saturating_sub(1))
        .map(|w| w + 2);
    let tail = headers.last().map_or(0, |h| h.chars().count());
    let total = columns.sum::<usize>() + tail;

    println!("{}", paint(dim(), &"╌".repeat(total)));
}
