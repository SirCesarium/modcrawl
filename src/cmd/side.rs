//! The `side` command.

use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde::Serialize;
use serde_json::to_string_pretty;

use crate::cmd::{Source, collect_sources};
use crate::core::detect_mod_type::ModType;
use crate::core::side::{Side, SideReport, side_report, side_report_reader};
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `side` command.
#[derive(ClapArgs)]
pub struct Args {
    /// Output the side report as JSON instead of a table
    #[arg(short, long)]
    pub json: bool,

    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,
}

/// One JSON entry: `{ "file", "type", "side" }`.
#[derive(Serialize)]
struct Entry {
    file: String,
    #[serde(rename = "type")]
    kind: ModType,
    side: Side,
}

/// Run the `side` command.
///
/// # Errors
///
/// Returns an error if an archive cannot be read or the report cannot be
/// serialized.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path)?;
    let mut entries = Vec::new();
    let mut rows = Vec::new();
    let mut failed = false;

    for source in &sources {
        match classify_source(source) {
            Ok(report) => {
                if args.json {
                    entries.push(Entry {
                        file: source.label(),
                        kind: report.mod_type,
                        side: report.side,
                    });
                } else {
                    rows.push((source.label(), report.mod_type, report.side));
                }
            }
            Err(e) => {
                ui::file_error(&source.label(), &e);
                failed = true;
            }
        }
    }

    if args.json {
        println!("{}", to_string_pretty(&entries)?);
    } else if !rows.is_empty() {
        ui::render_side(&rows);
    }

    if failed {
        return Err(Error::Failed);
    }

    Ok(())
}

/// Detect a source's type and classify its side.
fn classify_source(source: &Source) -> Result<SideReport> {
    match source {
        Source::Path(path) => side_report(path),
        Source::Archive(data) => {
            let mut cursor = Cursor::new(data.as_slice());
            side_report_reader(&mut cursor)
        }
    }
}
