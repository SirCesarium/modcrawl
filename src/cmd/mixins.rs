use std::collections::BTreeSet;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::collect_jar_paths;
use crate::core::classfile;
use crate::error::Result;
use crate::ui;

#[derive(ClapArgs)]
pub struct Args {
    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,

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
    let files = collect_jar_paths(&args.path)?;
    let multi = files.len() > 1;
    let mut total_entries = 0usize;

    for file in &files {
        let entries = match classfile::mixins(file) {
            Ok(e) => e,
            Err(e) => {
                ui::file_error(&file.display().to_string(), &e);
                continue;
            }
        };

        if args.json {
            println!("{}", to_string_pretty(&entries)?);
            continue;
        }

        if entries.is_empty() {
            continue;
        }

        total_entries += entries.len();

        if multi {
            ui::file_header(&file.display().to_string());
        }

        if args.quiet {
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for e in &entries {
                seen.insert(e.mixin_class.as_str());
            }

            for class_file in seen {
                println!("{class_file}");
            }
        } else {
            ui::render_mixins(&entries);
        }
    }

    if total_entries == 0 && !files.is_empty() && !args.json {
        ui::no_results("No mixins found.");
    }

    Ok(())
}
