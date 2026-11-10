//! The `grep` command.

use std::collections::BTreeSet;
use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{Source, collect_sources};
use crate::core::classfile;
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `grep` command.
#[derive(ClapArgs)]
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

    #[arg(
        long,
        help = "Fail if any class file cannot be parsed instead of skipping it"
    )]
    pub strict: bool,
}

/// Run the `grep` command.
///
/// # Errors
///
/// Returns an error if a JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path)?;
    let multi = sources.len() > 1;
    let mut total_matches = 0usize;
    let mut all_matches = Vec::new();
    let mut skipped = 0usize;
    let mut failed = false;

    for source in &sources {
        let scan = match source {
            Source::Path(path) => classfile::scan_grep(path, &args.pattern),
            Source::Archive(data) => {
                classfile::scan_grep_reader(&mut Cursor::new(data.as_slice()), &args.pattern)
            }
        };
        let scan = match scan {
            Ok(scan) => scan,
            Err(e) => {
                ui::file_error(&source.label(), &e);
                failed = true;
                continue;
            }
        };

        if !scan.skipped.is_empty() {
            ui::skipped_classes(&source.label(), &scan.skipped);
            skipped += scan.skipped.len();
        }
        let matches = scan.items;

        if args.json {
            all_matches.extend(matches);
            continue;
        }

        if matches.is_empty() {
            continue;
        }

        total_matches += matches.len();

        if multi {
            ui::file_header(&source.label());
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

    if args.json {
        println!("{}", to_string_pretty(&all_matches)?);
    } else if total_matches == 0 && !sources.is_empty() && !failed {
        ui::no_results(&format!(
            "No matches found for pattern \"{}\".",
            args.pattern
        ));
    }

    if failed {
        return Err(Error::Failed);
    }

    if args.strict && skipped > 0 {
        return Err(Error::ClassFile(format!(
            "{skipped} class file(s) could not be parsed"
        )));
    }

    Ok(())
}
