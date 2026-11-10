//! The `classes` command.

use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{ScanArgs, Source, collect_sources};
use crate::core::classfile;
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `classes` command.
#[derive(ClapArgs)]
pub struct Args {
    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,

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

    #[command(flatten)]
    pub scan: ScanArgs,
}

/// Run the `classes` command.
///
/// # Errors
///
/// Returns an error if a JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path, &args.scan.options())?;
    let multi = sources.len() > 1;
    let mut all_entries = Vec::new();
    let mut skipped = 0usize;
    let mut failed = false;

    for source in &sources {
        let scan = match source {
            Source::Path(path) => classfile::scan_classes(path),
            Source::Archive(data) => {
                classfile::scan_classes_reader(&mut Cursor::new(data.as_slice()))
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

        if multi {
            ui::file_header(&source.label());
        }

        ui::render_classes(&entries);
    }

    if args.json {
        println!("{}", to_string_pretty(&all_entries)?);
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::env;
    use std::fs;
    use std::io::{Cursor, Write};
    use std::process;

    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;

    fn malformed_jar() -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file("com/example/Broken.class", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"not a class file").unwrap();
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn strict_mode_fails_when_a_class_cannot_be_parsed() {
        let path = env::temp_dir().join(format!("modcrawl-strict-{}.jar", process::id()));
        fs::write(&path, malformed_jar()).unwrap();

        let args = Args {
            path: vec![path.clone()],
            json: false,
            strict: true,
            scan: ScanArgs::default(),
        };
        let result = run(&args);

        fs::remove_file(&path).unwrap();

        assert!(matches!(result, Err(Error::ClassFile(_))));
    }
}
