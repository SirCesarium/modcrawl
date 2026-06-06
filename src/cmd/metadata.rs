use std::path::PathBuf;

use crate::core::metadata::read_metadata;
use crate::error::Result;

#[derive(clap::Args)]
pub struct Args {
    /// Output compact JSON (one line) instead of pretty-printed
    #[arg(short, long)]
    pub json: bool,

    /// Path to the mod/plugin JAR file
    pub file: PathBuf,
}

/// Run the `metadata` command.
///
/// # Errors
///
/// Returns an error if the archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let meta = read_metadata(&args.file)?;
    if args.json {
        println!("{}", serde_json::to_string(&meta)?);
    } else {
        println!("{meta}");
    }
    Ok(())
}
