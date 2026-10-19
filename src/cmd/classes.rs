use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::collect_jar_paths;
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
    let files = collect_jar_paths(&args.path)?;
    let multi = files.len() > 1;

    for file in &files {
        let entries = match classfile::list_classes(file) {
            Ok(e) => e,
            Err(e) => {
                ui::file_error(&file.display().to_string(), &e);
                continue;
            }
        };

        if args.json {
            println!("{}", to_string_pretty(&entries)?);
            continue;
        }

        if multi {
            ui::file_header(&file.display().to_string());
        }

        ui::render_classes(&entries);
    }

    Ok(())
}
