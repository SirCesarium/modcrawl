use std::path::PathBuf;

use crate::core::metadata::read_metadata;
use crate::error::Result;
use crate::ui;

#[derive(clap::Args)]
pub struct Args {
    /// Output compact JSON (one line) instead of pretty-printed
    #[arg(short, long)]
    pub json: bool,

    /// Paths to the mod/plugin JAR files
    pub file: Vec<PathBuf>,
}

/// Run the `metadata` command.
///
/// # Errors
///
/// Returns an error if an archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let multi = args.file.len() > 1;

    for file in &args.file {
        let meta = match read_metadata(file) {
            Ok(m) => m,
            Err(e) => {
                ui::file_error(&file.display().to_string(), &e);
                continue;
            }
        };

        if args.json {
            println!("{}", serde_json::to_string(&meta)?);
            continue;
        }

        if multi {
            ui::file_header(&file.display().to_string());
        }

        ui::render_metadata(&meta);
    }

    Ok(())
}
