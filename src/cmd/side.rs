//! The `side` command.

use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde::Serialize;
use serde_json::to_string_pretty;

use crate::cmd::{Source, collect_sources};
use crate::core::detect_mod_type::ModType;
use crate::core::identify::{identify, identify_reader};
use crate::core::metadata::{read_metadata, read_metadata_reader};
use crate::core::side::{Side, classify};
use crate::error::Result;
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
    kind: String,
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

    for source in &sources {
        match classify_source(source) {
            Ok((kind, side)) => {
                if args.json {
                    entries.push(Entry {
                        file: source.label(),
                        kind: kind.to_string(),
                        side,
                    });
                } else {
                    rows.push((source.label(), kind, side));
                }
            }
            Err(e) => ui::file_error(&source.label(), &e),
        }
    }

    if args.json {
        println!("{}", to_string_pretty(&entries)?);
    } else {
        ui::render_side(&rows);
    }

    Ok(())
}

/// Detect a source's type and classify its side.
fn classify_source(source: &Source) -> Result<(ModType, Side)> {
    match source {
        Source::Path(path) => {
            let kind = identify(path)?;
            if kind == ModType::Unknown {
                return Ok((kind, Side::Unknown));
            }

            let metadata = read_metadata(path)?;
            Ok((kind, classify(&metadata)))
        }
        Source::Archive(data) => {
            let mut cursor = Cursor::new(data.as_slice());
            let kind = identify_reader(&mut cursor)?;
            if kind == ModType::Unknown {
                return Ok((kind, Side::Unknown));
            }

            let mut cursor = Cursor::new(data.as_slice());
            let metadata = read_metadata_reader(&mut cursor)?;
            Ok((kind, classify(&metadata)))
        }
    }
}
