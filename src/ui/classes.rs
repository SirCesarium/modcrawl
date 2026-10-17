//! `modcrawl classes` — the class listing table.

use anstream::println;
use anstyle::Style;

use super::lines::no_results;
use super::style::{dim, paint};
use super::table::{cell, print_table, styled_cell};
use crate::core::classfile::ClassEntry;

/// Render the list of classes found in a JAR.
pub(crate) fn render(classes: &[ClassEntry]) {
    if classes.is_empty() {
        no_results("No classes found.");
        return;
    }

    println!("{}", paint(Style::new().bold(), "Classes"));
    println!();

    let rows: Vec<_> = classes
        .iter()
        .map(|c| {
            vec![
                cell(&c.name),
                styled_cell(dim(), &c.java_version),
                styled_cell(dim(), &c.access_flags.join(" ")),
            ]
        })
        .collect();

    print_table(&["CLASS", "JAVA", "ACCESS"], &rows);
}
