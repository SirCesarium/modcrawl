//! The `metadata` command.

use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{ScanArgs, Source, collect_sources};
use crate::core::metadata::{read_metadata, read_metadata_reader};
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `metadata` command.
#[derive(ClapArgs)]
pub struct Args {
    /// Output as JSON instead of human-readable text
    #[arg(short, long)]
    pub json: bool,

    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,

    #[command(flatten)]
    pub scan: ScanArgs,
}

/// Run the `metadata` command.
///
/// # Errors
///
/// Returns an error if an archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path, &args.scan.options())?;
    let multi = sources.len() > 1;
    let mut entries = Vec::new();
    let mut failed = false;

    for source in &sources {
        let meta = match source {
            Source::Path(path) => read_metadata(path),
            Source::Archive(data) => read_metadata_reader(&mut Cursor::new(data.as_slice())),
        };
        let meta = match meta {
            Ok(m) => m,
            Err(e) => {
                ui::file_error(&source.label(), &e);
                failed = true;
                continue;
            }
        };

        if args.json {
            entries.push(meta);
            continue;
        }

        if multi {
            ui::file_header(&source.label());
        }

        ui::render_metadata(&meta);
    }

    if args.json {
        if multi {
            println!("{}", to_string_pretty(&entries)?);
        } else if let Some(meta) = entries.first() {
            println!("{}", to_string_pretty(meta)?);
        }
    }

    if failed {
        return Err(Error::Failed);
    }

    Ok(())
}
