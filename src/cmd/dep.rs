//! The `dep` command.

use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{ScanArgs, Source, collect_sources};
use crate::core::dep;
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `dep` command.
#[derive(ClapArgs)]
pub struct Args {
    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,

    #[arg(
        long = "include-jar-in-jar",
        alias = "jij",
        help = "Include embedded dependencies (JAR-in-JAR) in dependency report"
    )]
    pub include_jar_in_jar: bool,

    #[arg(
        short = 'j',
        long,
        help = "Output dependencies as JSON instead of human-readable text"
    )]
    pub json: bool,

    #[command(flatten)]
    pub scan: ScanArgs,
}

/// Run the `dep` command.
///
/// # Errors
///
/// Returns an error if an archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path, &args.scan.options())?;
    let multi = sources.len() > 1;
    let mut reports = Vec::new();
    let mut failed = false;

    for source in &sources {
        let report = match source {
            Source::Path(path) => dep::analyze(path, args.include_jar_in_jar),
            Source::Archive(data) => {
                dep::analyze_reader(&mut Cursor::new(data.as_slice()), args.include_jar_in_jar)
            }
        };
        let report = match report {
            Ok(r) => r,
            Err(e) => {
                ui::file_error(&source.label(), &e);
                failed = true;
                continue;
            }
        };

        if args.json {
            reports.push(report);
            continue;
        }

        if multi {
            ui::file_header(&source.label());
        }

        ui::render_deps(&report, args.include_jar_in_jar);
    }

    if args.json {
        if multi {
            println!("{}", to_string_pretty(&reports)?);
        } else if let Some(report) = reports.first() {
            println!("{}", to_string_pretty(report)?);
        }
    }

    if failed {
        return Err(Error::Failed);
    }

    Ok(())
}
