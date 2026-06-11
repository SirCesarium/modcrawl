use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::core::classfile;
use crate::error::Result;
use crate::GrepMatch;

#[derive(clap::Args)]
pub struct Args {
    pub pattern: String,

    pub file: Vec<PathBuf>,

    #[arg(
        short = 'q',
        long,
        help = "Only print class file names that contain matches"
    )]
    pub quiet: bool,

    #[arg(
        short = 'j',
        long,
        help = "Output as JSON instead of human-readable text"
    )]
    pub json: bool,
}

/// Run the `grep` command.
///
/// # Errors
///
/// Returns an error if a JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let mut total_matches = 0usize;

    for file in &args.file {
        let matches = match classfile::grep(file, &args.pattern) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}: {e:#}", file.display());
                continue;
            }
        };

        if args.json {
            println!("{}", serde_json::to_string_pretty(&matches)?);
            continue;
        }

        if matches.is_empty() {
            continue;
        }

        total_matches += matches.len();

        if args.quiet {
            if args.file.len() > 1 {
                println!("{}:", file.display());
            }
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for m in &matches {
                seen.insert(m.class_file.as_str());
            }
            for class_file in seen {
                println!("  {class_file}");
            }
        } else {
            if args.file.len() > 1 {
                println!("{}:", file.display());
            }
            let mut grouped: BTreeMap<&str, Vec<&GrepMatch>> = BTreeMap::new();
            for m in &matches {
                grouped.entry(m.class_file.as_str()).or_default().push(m);
            }
            for (class_file, class_matches) in &grouped {
                println!("  {class_file}:");
                for m in class_matches {
                    println!("    {}: \"{}\"", m.pool_tag, m.value);
                }
            }
        }
    }

    if total_matches == 0 && !args.file.is_empty() {
        let n = args.file.len();
        eprintln!(
            "No matches found for pattern \"{}\" in {n} file(s).",
            args.pattern
        );
    }

    Ok(())
}
