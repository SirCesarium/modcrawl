use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::core::classfile;
use crate::error::Result;
use crate::MixinEntry;

#[derive(clap::Args)]
pub struct Args {
    pub file: Vec<PathBuf>,

    #[arg(
        short = 'q',
        long,
        help = "Only print mixin class names that contain matches"
    )]
    pub quiet: bool,

    #[arg(
        short = 'j',
        long,
        help = "Output as JSON instead of human-readable text"
    )]
    pub json: bool,
}

/// Run the `mixins` command.
///
/// # Errors
///
/// Returns an error if a JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    for file in &args.file {
        let entries = match classfile::mixins(file) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{}: {e:#}", file.display());
                continue;
            }
        };

        if args.json {
            println!("{}", serde_json::to_string_pretty(&entries)?);
            continue;
        }

        if entries.is_empty() {
            continue;
        }

        if args.quiet {
            if args.file.len() > 1 {
                println!("{}:", file.display());
            }
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for e in &entries {
                seen.insert(e.mixin_class.as_str());
            }
            for class_file in seen {
                println!("  {class_file}");
            }
        } else {
            if args.file.len() > 1 {
                println!("{}:", file.display());
            }
            let mut grouped: BTreeMap<&str, Vec<&MixinEntry>> = BTreeMap::new();
            for e in &entries {
                grouped.entry(e.mixin_class.as_str()).or_default().push(e);
            }
            for (class_file, class_entries) in &grouped {
                println!("  {class_file} ->");
                for e in class_entries {
                    println!("    {}", e.target);
                }
            }
        }
    }
    Ok(())
}
