use std::path::PathBuf;

use crate::core::classfile;
use crate::error::Result;
use crate::ui;

#[derive(clap::Args)]
pub struct Args {
    pub file: Vec<PathBuf>,

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
    let multi = args.file.len() > 1;

    for file in &args.file {
        let entries = match classfile::list_classes(file) {
            Ok(e) => e,
            Err(e) => {
                ui::file_error(&file.display().to_string(), &e);
                continue;
            }
        };

        if args.json {
            println!("{}", serde_json::to_string_pretty(&entries)?);
            continue;
        }

        if multi {
            ui::file_header(&file.display().to_string());
        }

        ui::render_classes(&entries);
    }

    Ok(())
}
