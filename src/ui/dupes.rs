//! `modcrawl dupes` — the duplicate-class tables.

use anstream::println;
use anstyle::Style;

use super::lines::no_results;
use super::style::{dim, paint};
use super::table::{cell, print_table, styled_cell};
use crate::core::classfile::{DupEntry, group_conflicts};

/// Render duplicates, grouped by class or by conflict pair.
pub(crate) fn render(entries: &[DupEntry], by_class: bool) {
    if entries.is_empty() {
        no_results("No duplicate classes found.");
        return;
    }

    if by_class {
        render_by_class(entries);
    } else {
        render_conflicts(entries);
    }
}

/// `CLASS / FILES` — one row per duplicated class.
fn render_by_class(entries: &[DupEntry]) {
    println!("{}", paint(Style::new().bold(), "Duplicate classes"));
    println!();

    let rows: Vec<_> = entries
        .iter()
        .map(|d| vec![cell(&d.class_name), styled_cell(dim(), &d.files.join(", "))])
        .collect();

    print_table(&["CLASS", "FILES"], &rows);
}

/// `FILES / SHARED CLASSES` — one row per pair of JARs that share classes.
fn render_conflicts(entries: &[DupEntry]) {
    let pairs = group_conflicts(entries);

    if pairs.is_empty() {
        no_results("No duplicate classes found.");
        return;
    }

    println!("{}", paint(Style::new().bold(), "Classpath conflicts"));
    println!();

    let rows: Vec<_> = pairs
        .iter()
        .map(|e| {
            vec![
                cell(&format!("{}  <->  {}", e.file_a, e.file_b)),
                styled_cell(dim(), &e.classes.join(", ")),
            ]
        })
        .collect();

    print_table(&["FILES", "SHARED CLASSES"], &rows);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflicts_never_pair_a_file_with_itself() {
        let entry = DupEntry {
            class_name: String::from("com/example/Shared.class"),
            files: vec![String::from("foo.jar"), String::from("foo.jar")],
        };

        assert!(group_conflicts(&[entry]).is_empty());
    }
}
