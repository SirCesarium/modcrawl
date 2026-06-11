use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use crate::core::classfile;
use crate::error::Result;

#[derive(clap::Args)]
pub struct Args {
    /// One or more JAR files to compare for duplicate classes.
    pub file: Vec<PathBuf>,

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

fn print_conflict_pairs(entries: &[classfile::DupEntry]) {
    let mut conflicts: BTreeMap<(String, String), Vec<&str>> = BTreeMap::new();

    for entry in entries {
        let files = &entry.files;
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let (a, b) = if files[i] < files[j] {
                    (files[i].as_str(), files[j].as_str())
                } else {
                    (files[j].as_str(), files[i].as_str())
                };
                conflicts
                    .entry((a.to_string(), b.to_string()))
                    .or_default()
                    .push(&entry.class_name);
            }
        }
    }

    if conflicts.is_empty() {
        return;
    }

    println!("── Classpath Conflicts ──");
    println!();

    for ((file_a, file_b), classes) in &conflicts {
        println!("  {file_a}");
        println!("  {file_b}");
        println!(
            "  ── {} shared {} ──",
            classes.len(),
            if classes.len() == 1 {
                "class"
            } else {
                "classes"
            }
        );
        for class_name in classes {
            println!("    {class_name}");
        }
        println!();
    }
}

fn print_by_class(entries: &[classfile::DupEntry]) {
    for entry in entries {
        println!(
            "  {}  (in {} {})",
            entry.class_name,
            entry.files.len(),
            if entry.files.len() == 1 {
                "JAR"
            } else {
                "JARs"
            }
        );
        for file in &entry.files {
            println!("    {file}");
        }
    }
}

/// Run the `dupes` command.
///
/// # Errors
///
/// Returns an error if any JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let paths: Vec<&Path> = args.file.iter().map(PathBuf::as_path).collect();
    let entries = classfile::find_duplicates(&paths)?;

    if args.json && args.count {
        let total_files: usize = entries
            .iter()
            .flat_map(|e| &e.files)
            .collect::<BTreeSet<_>>()
            .len();
        println!(
            "{}",
            serde_json::json!({
                "total_duplicate_classes": entries.len(),
                "total_files_with_dupes": total_files,
            })
        );
        return Ok(());
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
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
        println!("No duplicate classes found.");
        return Ok(());
    }

    if args.by_class {
        print_by_class(&entries);
    } else {
        print_conflict_pairs(&entries);
    }

    Ok(())
}
