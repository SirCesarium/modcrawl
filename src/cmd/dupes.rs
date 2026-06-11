use std::path::Path;
use std::path::PathBuf;

use crate::core::classfile;
use crate::error::Result;

#[derive(clap::Args)]
pub struct Args {
    /// One or more JAR files to compare for duplicate classes.
    pub file: Vec<PathBuf>,

    #[arg(
        short = 'j',
        long,
        help = "Output as JSON instead of human-readable text"
    )]
    pub json: bool,
}

/// Run the `dupes` command.
///
/// # Errors
///
/// Returns an error if any JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let paths: Vec<&Path> = args.file.iter().map(PathBuf::as_path).collect();
    let entries = classfile::find_duplicates(&paths)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    for entry in &entries {
        println!("  {}:", entry.class_name);
        for file in &entry.files {
            println!("    {file}");
        }
    }

    Ok(())
}
