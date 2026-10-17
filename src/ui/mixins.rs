//! `modcrawl mixins` — the mixin/target table.

use anstream::println;
use anstyle::Style;

use super::lines::no_results;
use super::style::{dim, paint};
use super::table::{cell, print_table, styled_cell};
use crate::core::classfile::MixinEntry;

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
