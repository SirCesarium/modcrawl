use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::{json, to_string_pretty};

use crate::cmd::collect_jar_paths;
use crate::core::classfile;
use crate::error::Result;
use crate::ui;

#[derive(ClapArgs)]
pub struct Args {
    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,

    #[arg(
        short = 'b',
        long,
        help = "Group duplicates by class instead of by conflict pair"
    )]
    pub by_class: bool,

    #[arg(
        short = 'c',
        long,
        help = "Show only a count of duplicate classes instead of the full listing"
    )]
    pub count: bool,

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
    let paths = collect_jar_paths(&args.path)?;
    let paths: Vec<&Path> = paths.iter().map(PathBuf::as_path).collect();
    let entries = classfile::find_duplicates(&paths)?;

    if args.json && args.count {
        let total_files: usize = entries
            .iter()
            .flat_map(|e| &e.files)
            .collect::<BTreeSet<_>>()
            .len();
        println!(
            "{}",
            json!({
                "total_duplicate_classes": entries.len(),
                "total_files_with_dupes": total_files,
            })
        );
        return Ok(());
    }

    if args.json {
        println!("{}", to_string_pretty(&entries)?);
        return Ok(());
    }

    if args.count {
        let total_files: usize = entries
            .iter()
            .flat_map(|e| &e.files)
            .collect::<BTreeSet<_>>()
            .len();
        println!(
            "Found {} duplicate classes across {} JAR files.",
            entries.len(),
            total_files
        );
        return Ok(());
    }

    if entries.is_empty() {
        ui::no_results("No duplicate classes found.");
        return Ok(());
    }

    ui::render_dupes(&entries, args.by_class);

    Ok(())
}
