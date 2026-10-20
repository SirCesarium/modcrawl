use std::collections::BTreeSet;
use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{Source, collect_sources};
use crate::core::classfile;
use crate::error::Result;
use crate::ui;

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

    for source in &sources {
        let matches = match source {
            Source::Path(path) => classfile::grep(path, &args.pattern),
            Source::Archive(data) => {
                classfile::grep_reader(&mut Cursor::new(data.as_slice()), &args.pattern)
            }
        };
        let matches = match matches {
            Ok(m) => m,
            Err(e) => {
                ui::file_error(&source.label(), &e);
                continue;
            }
        };

        if args.json {
            println!("{}", to_string_pretty(&matches)?);
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

    if total_matches == 0 && !sources.is_empty() && !args.json {
        ui::no_results(&format!(
            "No matches found for pattern \"{}\".",
            args.pattern
        ));
    }

    Ok(())
}
