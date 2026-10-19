use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::cmd::collect_jar_paths;
use crate::core::classfile;
use crate::error::Result;
use crate::ui;

#[derive(clap::Args)]
pub struct Args {
    pub pattern: String,

    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,

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
    let multi = args.path.len() > 1;
    let files = collect_jar_paths(&args.path)?;
    let mut total_matches = 0usize;

    for file in &files {
        let matches = match classfile::grep(file, &args.pattern) {
            Ok(m) => m,
            Err(e) => {
                ui::file_error(&file.display().to_string(), &e);
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

        if multi {
            ui::file_header(&file.display().to_string());
        }

        if args.quiet {
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for m in &matches {
                seen.insert(m.class_file.as_str());
            }

            for class_file in seen {
                println!("{class_file}");
            }
        } else {
            ui::render_grep(&matches);
        }
    }

    if total_matches == 0 && !args.path.is_empty() {
        ui::no_results(&format!(
            "No matches found for pattern \"{}\".",
            args.pattern
        ));
    }

    Ok(())
}
