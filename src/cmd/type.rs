use std::path::PathBuf;

use crate::error::Result;

#[derive(clap::Args)]
pub struct Args {
    pub file: PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    println!("type command: file={}", args.file.display());
    Ok(())
}
