use std::path::PathBuf;

use crate::core::metadata::read_metadata;
use crate::error::Result;

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
        let result = read_metadata(file);
        let meta = match result {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}: {e:#}", file.display());
                continue;
            }
        };
        if args.json {
            println!("{}", serde_json::to_string(&meta)?);
        } else {
            if multi {
                println!("{}:", file.display());
            }
            println!("{meta}");
            if multi {
                println!();
            }
        }
    }
    Ok(())
}
