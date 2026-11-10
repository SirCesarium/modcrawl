//! The `mixins` command.

use std::collections::BTreeSet;
use std::io::Cursor;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde_json::to_string_pretty;

use crate::cmd::{ScanArgs, Source, collect_sources};
use crate::core::classfile::{self, group_mixin_overlaps};
use crate::error::{Error, Result};
use crate::ui;

/// Arguments for the `mixins` command.
#[derive(ClapArgs)]
#[allow(clippy::struct_excessive_bools)]
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
        short = 'o',
        long,
        help = "Report targets mixed into by more than one JAR (requires multiple JARs)"
    )]
    pub overlaps: bool,

    #[arg(
        long,
        help = "Fail if any class file cannot be parsed instead of skipping it"
    )]
    pub strict: bool,

    #[command(flatten)]
    pub scan: ScanArgs,
}

/// Run the `mixins` command.
///
/// # Errors
///
/// Returns an error if a JAR cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let sources = collect_sources(&args.path, &args.scan.options())?;
    let multi = sources.len() > 1;
    let mut total_entries = 0usize;
    // (jar_label, MixinEntry) pairs accumulated for overlap detection.
    let mut all_labelled: Vec<(String, classfile::MixinEntry)> = Vec::new();
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

        if args.overlaps || args.json {
            for entry in &entries {
                all_labelled.push((source.label(), entry.clone()));
            }
        }

        if args.json || args.overlaps {
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

    if args.overlaps {
        let overlaps = group_mixin_overlaps(&all_labelled);
        if args.json {
            println!("{}", to_string_pretty(&overlaps)?);
        } else if args.quiet {
            for o in &overlaps {
                println!("{}", o.target);
            }
        } else {
            ui::render_mixin_overlaps(&overlaps);
        }
    } else if args.json {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::env;
    use std::fs;
    use std::io::Write;
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
        let path = env::temp_dir().join(format!("modcrawl-mixins-strict-{}.jar", process::id()));
        fs::write(&path, malformed_jar()).unwrap();

        let args = Args {
            path: vec![path.clone()],
            quiet: false,
            json: false,
            overlaps: false,
            strict: true,
            scan: ScanArgs::default(),
        };
        let result = run(&args);

        fs::remove_file(&path).unwrap();

        assert!(matches!(result, Err(Error::ClassFile(_))));
    }

    #[test]
    fn overlaps_flag_succeeds_with_empty_results() {
        let path = env::temp_dir().join(format!("modcrawl-mixins-empty-{}.jar", process::id()));
        let zip = ZipWriter::new(Cursor::new(Vec::new()));
        let bytes = zip.finish().unwrap().into_inner();
        fs::write(&path, bytes).unwrap();

        let args = Args {
            path: vec![path.clone()],
            quiet: false,
            json: true,
            overlaps: true,
            strict: false,
            scan: ScanArgs::default(),
        };
        assert!(run(&args).is_ok());

        fs::remove_file(&path).unwrap();
    }
}
