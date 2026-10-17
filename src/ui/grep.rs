//! `modcrawl grep` — the constant-pool match table.

use anstream::println;
use anstyle::Style;

use super::lines::no_results;
use super::style::{dim, paint};
use super::table::{cell, print_table, styled_cell};
use crate::core::classfile::GrepMatch;

/// Render the constant-pool entries matching the needle.
pub(crate) fn render(matches: &[GrepMatch]) {
    if matches.is_empty() {
        no_results("No matches found.");
        return;
    }

    println!("{}", paint(Style::new().bold(), "Constant pool matches"));
    println!();

    let rows: Vec<_> = matches
        .iter()
        .map(|m| {
            vec![
                cell(&m.class_file),
                styled_cell(dim(), &m.pool_tag),
                cell(&m.value),
            ]
        })
        .collect();

    print_table(&["CLASS", "POOL", "VALUE"], &rows);
}
