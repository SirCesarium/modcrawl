//! `modcrawl dep` — the dependency report.

use anstream::println;

use super::lines::no_results;
use super::style::{dim, kind_label, kind_style};
use super::table::{cell, print_table, styled_cell};
use crate::core::dep::types::DepReport;

/// Render one JAR's dependency report.
pub(crate) fn render(report: &DepReport, include_jar_in_jar: bool) {
    let has_jars = include_jar_in_jar && !report.jar_in_jar.is_empty();

    if report.dependencies.is_empty() && !has_jars {
        no_results("No dependencies found.");
        return;
    }

    render_dependencies(report);
    println!();

    if has_jars {
        render_jar_in_jar(report);
    }
}

/// The `MOD / KIND / VERSION` table.
fn render_dependencies(report: &DepReport) {
    let rows: Vec<_> = report
        .dependencies
        .iter()
        .map(|dep| {
            vec![
                cell(&dep.name),
                styled_cell(kind_style(dep.kind), kind_label(dep.kind)),
                styled_cell(dim(), &dep.version_range.to_string()),
            ]
        })
        .collect();

    print_table(&["MOD", "KIND", "VERSION"], &rows);
}

/// The `EMBEDDED JAR` table for jar-in-jar payloads.
fn render_jar_in_jar(report: &DepReport) {
    let rows: Vec<_> = report
        .jar_in_jar
        .iter()
        .map(|jar| vec![cell(&jar.path)])
        .collect();

    print_table(&["EMBEDDED JAR"], &rows);
}
