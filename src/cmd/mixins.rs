//! The `mixins` command.

use std::collections::BTreeSet;
use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{Source, collect_sources};
use crate::core::classfile;
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `mixins` command.
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

    #[arg(
        long,
        help = "Fail if any class file cannot be parsed instead of skipping it"
    )]
    pub strict: bool,
}

/// Run the `mixins` command.
///
/// # Errors
///
/// Returns an error if a JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path)?;
    let multi = sources.len() > 1;
    let mut total_entries = 0usize;
    let mut all_entries = Vec::new();
    let mut skipped = 0usize;
    let mut failed = false;

    for source in &sources {
        let scan = match source {
            Source::Path(path) => classfile::scan_mixins(path),
            Source::Archive(data) => {
                classfile::scan_mixins_reader(&mut Cursor::new(data.as_slice()))
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
        let entries = scan.items;

        if args.json {
            all_entries.extend(entries);
            continue;
        }

        if entries.is_empty() {
            continue;
        }

        total_entries += entries.len();

        if multi {
            ui::file_header(&source.label());
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

    if args.json {
        println!("{}", to_string_pretty(&all_entries)?);
    } else if total_entries == 0 && !sources.is_empty() && !failed {
        ui::no_results("No mixins found.");
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
