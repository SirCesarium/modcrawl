//! The `dupes` command.

use std::collections::BTreeSet;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::{json, to_string_pretty};

use crate::cmd::collect_sources;
use crate::core::classfile::{ClassIndex, group_conflicts};
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `dupes` command.
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
    let sources = collect_sources(&args.path)?;

    let mut paths = Vec::with_capacity(sources.len());
    for source in &sources {
        match source.as_path() {
            Some(path) => paths.push(path),
            None => {
                return Err(Error::Usage(String::from(
                    "dupes compares several JARs: pass their paths, or pipe a list of paths",
                )));
            }
        }
    }

    let mut index = ClassIndex::default();
    let mut failed = false;

    for path in &paths {
        if let Err(e) = index.add_path(path) {
            ui::file_error(&path.display().to_string(), &e);
            failed = true;
        }
    }

    let entries = index.into_duplicates();

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
    } else if args.json && args.by_class {
        println!("{}", to_string_pretty(&entries)?);
    } else if args.json {
        println!("{}", to_string_pretty(&group_conflicts(&entries))?);
    } else if args.count {
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
    } else if entries.is_empty() {
        if !failed {
            ui::no_results("No duplicate classes found.");
        }
    } else {
        ui::render_dupes(&entries, args.by_class);
    }

    if failed {
        return Err(Error::Failed);
    }

    Ok(())
}
