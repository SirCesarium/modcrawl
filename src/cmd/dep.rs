use std::path::PathBuf;

use crate::core::dep;
use crate::error::Result;

#[derive(clap::Args)]
pub struct Args {
    pub file: Vec<PathBuf>,

    #[arg(
        long = "include-jar-in-jar",
        alias = "jij",
        help = "Include embedded dependencies (JAR-in-JAR) in dependency report"
    )]
    pub include_jar_in_jar: bool,

    #[arg(
        short = 'j',
        long,
        help = "Output dependencies as JSON instead of human-readable text"
    )]
    pub json: bool,
}

/// Run the `dep` command.
///
/// # Errors
///
/// Returns an error if an archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let multi = args.file.len() > 1;
    for file in &args.file {
        let result = dep::analyze(file, args.include_jar_in_jar);
        let report = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}: {e:#}", file.display());
                continue;
            }
        };
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            if multi {
                println!("{}:", file.display());
            }
            print!("{report}");
            if multi {
                println!();
            }
        }
    }
    Ok(())
}
