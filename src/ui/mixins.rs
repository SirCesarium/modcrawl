//! `modcrawl mixins` — the mixin/target table and cross-JAR overlap report.

use anstream::println;
use anstyle::Style;

use super::lines::no_results;
use super::style::{dim, paint};
use super::table::{cell, print_table, styled_cell};
use crate::core::classfile::{MixinEntry, MixinOverlap};

/// Render the mixins declared in a JAR and the classes they target.
pub(crate) fn render(mixins: &[MixinEntry]) {
    if mixins.is_empty() {
        no_results("No mixins found.");
        return;
    }

    println!("{}", paint(Style::new().bold(), "Mixins"));
    println!();

    let rows: Vec<_> = mixins
        .iter()
        .map(|m| vec![cell(&m.mixin_class), styled_cell(dim(), &m.target)])
        .collect();

    print_table(&["MIXIN", "TARGET"], &rows);
}

/// Render targets that are mixed into by classes from more than one JAR.
pub(crate) fn render_overlaps(overlaps: &[MixinOverlap]) {
    if overlaps.is_empty() {
        no_results("No overlapping mixin targets found.");
        return;
    }

    println!(
        "{}",
        paint(Style::new().bold(), "Overlapping mixin targets")
    );
    println!();

    let rows: Vec<_> = overlaps
        .iter()
        .flat_map(|o| {
            o.mixins.iter().map(|e| {
                vec![
                    cell(&o.target),
                    styled_cell(dim(), &e.jar),
                    styled_cell(dim(), &e.mixin_class),
                ]
            })
        })
        .collect();

    print_table(&["TARGET", "JAR", "MIXIN"], &rows);
}
