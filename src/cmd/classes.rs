use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{Source, collect_sources};
use crate::core::classfile;
use crate::error::Result;
use crate::ui;

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
                continue;
            }
        };

        if args.json {
            println!("{}", to_string_pretty(&entries)?);
            continue;
        }

        if multi {
            ui::file_header(&source.label());
        }

        ui::render_classes(&entries);
    }

    Ok(())
}
