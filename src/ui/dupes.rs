//! `modcrawl dupes` — the duplicate-class tables.

use std::collections::BTreeMap;

use anstream::println;
use anstyle::Style;

use super::lines::no_results;
use super::style::{dim, paint};
use super::table::{cell, print_table, styled_cell};
use crate::core::classfile::DupEntry;

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
        .map(|((a, b), classes)| {
            vec![
                cell(&format!("{a}  <->  {b}")),
                styled_cell(dim(), &classes.join(", ")),
            ]
        })
        .collect();

    print_table(&["FILES", "SHARED CLASSES"], &rows);
}

/// Pair up every combination of JARs that share a class, keyed by sorted pair.
fn group_conflicts(entries: &[DupEntry]) -> BTreeMap<(String, String), Vec<String>> {
    let mut conflicts: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    for entry in entries {
        for i in 0..entry.files.len() {
            for j in (i + 1)..entry.files.len() {
                let (a, b) = if entry.files[i] < entry.files[j] {
                    (entry.files[i].clone(), entry.files[j].clone())
                } else {
                    (entry.files[j].clone(), entry.files[i].clone())
                };

                conflicts
                    .entry((a, b))
                    .or_default()
                    .push(entry.class_name.clone());
            }
        }
    }

    conflicts
}
