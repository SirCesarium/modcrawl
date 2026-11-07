//! `modcrawl side` — the file/type/side table.

use super::style::{loader_name, loader_style, side_label, side_style};
use super::table::{cell, print_table, styled_cell};
use crate::core::detect_mod_type::ModType;
use crate::core::side::Side;

/// Render each `(file, type, side)` entry as a table row.
pub(crate) fn render(entries: &[(String, ModType, Side)]) {
    let rows: Vec<_> = entries
        .iter()
        .map(|(file, kind, side)| {
            vec![
                cell(file),
                styled_cell(loader_style(kind), loader_name(kind)),
                styled_cell(side_style(*side), side_label(*side)),
            ]
        })
        .collect();

    print_table(&["FILE", "TYPE", "SIDE"], &rows);
}
