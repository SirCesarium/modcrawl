use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string;

use crate::cmd::collect_jar_paths;
use crate::core::metadata::read_metadata;
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
    let files = collect_jar_paths(&args.path)?;
    let multi = files.len() > 1;

    for file in &files {
        let meta = match read_metadata(file) {
            Ok(m) => m,
            Err(e) => {
                ui::file_error(&file.display().to_string(), &e);
                continue;
            }
        };

        if args.json {
            println!("{}", to_string(&meta)?);
            continue;
        }

        if multi {
            ui::file_header(&file.display().to_string());
        }

        ui::render_metadata(&meta);
    }

    Ok(())
}
