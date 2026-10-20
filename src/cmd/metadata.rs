use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string;

use crate::cmd::{Source, collect_sources};
use crate::core::metadata::{read_metadata, read_metadata_reader};
use crate::error::Result;
use crate::ui;

#[derive(ClapArgs)]
pub struct Args {
    /// Output compact JSON (one line) instead of pretty-printed
    #[arg(short, long)]
    pub json: bool,

    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,
}

/// Run the `metadata` command.
///
/// # Errors
///
/// Returns an error if an archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path)?;
    let multi = sources.len() > 1;

    for source in &sources {
        let meta = match source {
            Source::Path(path) => read_metadata(path),
            Source::Archive(data) => read_metadata_reader(&mut Cursor::new(data.as_slice())),
        };
        let meta = match meta {
            Ok(m) => m,
            Err(e) => {
                ui::file_error(&source.label(), &e);
                continue;
            }
        };

        if args.json {
            println!("{}", to_string(&meta)?);
            continue;
        }

        if multi {
            ui::file_header(&source.label());
        }

        ui::render_metadata(&meta);
    }

    Ok(())
}
