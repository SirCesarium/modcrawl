//! The `classes` command.

use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{Source, collect_sources};
use crate::core::classfile;
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `classes` command.
#[derive(ClapArgs)]
pub struct Args {
    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,

    #[arg(
        short = 'j',
        long,
        help = "Output as JSON instead of human-readable text"
    )]
    pub json: bool,
}

/// Run the `classes` command.
///
/// # Errors
///
/// Returns an error if a JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path)?;
    let multi = sources.len() > 1;
    let mut all_entries = Vec::new();
    let mut failed = false;

    for source in &sources {
        let entries = match source {
            Source::Path(path) => classfile::list_classes(path),
            Source::Archive(data) => {
                classfile::list_classes_reader(&mut Cursor::new(data.as_slice()))
            }
        };
        let entries = match entries {
            Ok(e) => e,
            Err(e) => {
                ui::file_error(&source.label(), &e);
                failed = true;
                continue;
            }
        };

        if args.json {
            all_entries.extend(entries);
            continue;
        }

        if multi {
            ui::file_header(&source.label());
        }

        ui::render_classes(&entries);
    }

    if args.json {
        println!("{}", to_string_pretty(&all_entries)?);
    }

    if failed {
        return Err(Error::Failed);
    }

    Ok(())
}
