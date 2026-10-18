//! `modcrawl type` — the loader detection line.

use anstream::println;

use super::style::{loader_name, loader_style, paint};
use super::table::{cell, print_table, styled_cell};
use crate::core::detect_mod_type::ModType;

/// Render a single detected type on its own line, colored by loader.
pub(crate) fn render(kind: &ModType) {
    println!("{}", paint(loader_style(kind), loader_name(kind)));
}

/// Render one detected type from a concatenated stream, prefixed with its
/// position in the stream so a multi-ZIP blob stays readable.
pub(crate) fn render_tagged(index: usize, kind: &ModType) {
    println!("[{index}] {}", paint(loader_style(kind), loader_name(kind)));
}

/// Render many detected types as a `FILE / TYPE` table.
pub(crate) fn render_multi(entries: &[(String, ModType)]) {
    let rows: Vec<_> = entries
        .iter()
        .map(|(file, kind)| {
            vec![
                cell(file),
                styled_cell(loader_style(kind), loader_name(kind)),
            ]
        })
        .collect();

    print_table(&["FILE", "TYPE"], &rows);
}
