//! The `side` command.

use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde::Serialize;
use serde_json::to_string_pretty;

use crate::cmd::{ScanArgs, Source, collect_sources};
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

    #[command(flatten)]
    pub scan: ScanArgs,
}

/// One JSON entry: `{ "file", "side" }`.
#[derive(Serialize)]
struct Entry {
    file: String,
    side: Side,
}

/// Run the `side` command.
///
/// A run that addressed exactly one JAR prints its side alone; a directory or
/// a longer list prints a `FILE / SIDE` table.
///
/// # Errors
///
/// Returns an error if an archive cannot be read or the report cannot be
/// serialized.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path, &args.scan.options())?;
    let names_a_listing = matches!(args.path.as_slice(), [one] if one.is_dir());
    let mut entries = Vec::new();
    let mut rows = Vec::new();
    let mut failed = false;

    for source in &sources {
        match classify_source(source) {
            Ok(report) => {
                if args.json {
                    entries.push(Entry {
                        file: source.label(),
                        side: report.side,
                    });
                } else {
                    rows.push((source.label(), report.side));
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
    } else if let [single] = rows.as_slice()
        && !names_a_listing
    {
        ui::render_side_one(single.1);
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
