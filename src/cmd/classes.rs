use std::path::PathBuf;

use crate::core::classfile;
use crate::error::Result;

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
    for file in &args.file {
        let entries = match classfile::list_classes(file) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{}: {e:#}", file.display());
                continue;
            }
        };

        if args.json {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        } else {
            if args.file.len() > 1 {
                println!("{}:", file.display());
            }
            for entry in &entries {
                let flags = entry.access_flags.join(" ");
                println!("  {}  Java {}  {}", entry.name, entry.java_version, flags);
            }
        }
    }
    Ok(())
}
