//! `modcrawl side` — the side line and the file/side table.

use anstream::println;

use super::style::{paint, side_label, side_style};
use super::table::{cell, print_table, styled_cell};
use crate::core::side::Side;

/// Render one side on its own line, colored by side.
pub(crate) fn render_one(side: Side) {
    println!("{}", paint(side_style(side), side_label(side)));
}

/// Render each `(file, side)` entry as a table row.
pub(crate) fn render(entries: &[(String, Side)]) {
    let rows: Vec<_> = entries
        .iter()
        .map(|(file, side)| {
            vec![
                cell(file),
                styled_cell(side_style(*side), side_label(*side)),
            ]
        })
        .collect();

    print_table(&["FILE", "SIDE"], &rows);
}
