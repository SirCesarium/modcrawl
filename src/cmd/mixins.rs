//! The `mixins` command.

use std::collections::BTreeSet;
use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{Source, collect_sources};
use crate::core::classfile;
use crate::error::Result;
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

    for source in &sources {
        let entries = match source {
            Source::Path(path) => classfile::mixins(path),
            Source::Archive(data) => classfile::mixins_reader(&mut Cursor::new(data.as_slice())),
        };
        let entries = match entries {
            Ok(e) => e,
            Err(e) => {
                ui::file_error(&source.label(), &e);
                continue;
            }
        };

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
    } else if total_entries == 0 && !sources.is_empty() {
        ui::no_results("No mixins found.");
    }

    Ok(())
}
