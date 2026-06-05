use std::path::PathBuf;

use crate::error::Result;

#[derive(clap::Args)]
pub struct Args {
    pub file: PathBuf,
}

/// Run the `metadata` command.
///
/// # Errors
///
/// Returns an error if the archive cannot be read.
#[allow(clippy::unnecessary_wraps)]
pub fn run(args: &Args) -> Result<()> {
    println!("metadata command: file={}", args.file.display());
    Ok(())
}
