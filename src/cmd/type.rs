//! The `type` command.

use std::io;
use std::path::PathBuf;

use clap::Args as ClapArgs;
use serde::Serialize;
use serde_json::to_string_pretty;

use crate::cmd::{ScanArgs, Stdin, collect_jar_paths, is_stdin_request, read_stdin};
use crate::core::detect_mod_type::ModType;
use crate::core::identify::{identify, identify_reader};
use crate::error::{Error, Result};
use crate::ui;

const EOCD_MAGIC: &[u8] = b"PK\x05\x06";

/// Arguments for the `type` command.
#[derive(ClapArgs)]
pub struct Args {
    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,

    #[arg(
        short = 'j',
        long,
        help = "Output type as JSON instead of human-readable text"
    )]
    pub json: bool,

    #[command(flatten)]
    pub scan: ScanArgs,
}

/// A single entry in the JSON output of the `type` command.
#[derive(Serialize)]
struct TypeEntry {
    file: String,
    #[serde(rename = "type")]
    mod_type: ModType,
}

fn process_zip_chunks(data: &[u8], json: bool) -> Result<bool> {
    let mut cursor = io::Cursor::new(data);
    if let Ok(mod_type) = identify_reader(&mut cursor) {
        if json {
            println!(
                "{}",
                to_string_pretty(&TypeEntry {
                    file: "<stdin>".to_owned(),
                    mod_type,
                })?
            );
        } else {
            ui::render_type(&mod_type);
        }
        return Ok(false);
    }

    let eocd_positions: Vec<usize> = data
        .windows(4)
        .enumerate()
        .filter(|(_, w)| *w == EOCD_MAGIC)
        .map(|(i, _)| i)
        .collect();

    let mut entries: Vec<TypeEntry> = Vec::new();
    let mut n = 0usize;
    let mut prev = 0;

    for &eocd in &eocd_positions {
        if eocd + 22 > data.len() {
            continue;
        }
        let comment_len = u16::from_le_bytes([data[eocd + 20], data[eocd + 21]]) as usize;
        let chunk_end = eocd + 22 + comment_len;
        if chunk_end > data.len() || chunk_end <= prev {
            continue;
        }
        let chunk = &data[prev..chunk_end];
        let mut cursor = io::Cursor::new(chunk);
        if let Ok(mod_type) = identify_reader(&mut cursor) {
            n += 1;
            if json {
                entries.push(TypeEntry {
                    file: format!("<stdin:{n}>"),
                    mod_type,
                });
            } else {
                ui::render_type_tagged(n, &mod_type);
            }
        }
        prev = chunk_end;
    }

    if n == 0 {
        eprintln!("No valid ZIPs found in concatenated stream");
        return Ok(true);
    }

    if json {
        println!("{}", to_string_pretty(&entries)?);
    }

    Ok(false)
}

/// Collect and identify types for each JAR, returning entries or signalling failure.
fn collect_entries(files: &[PathBuf]) -> (Vec<TypeEntry>, bool) {
    let mut failed = false;
    let entries = files
        .iter()
        .filter_map(|file| match identify(file) {
            Ok(mod_type) => Some(TypeEntry {
                file: file.display().to_string(),
                mod_type,
            }),
            Err(e) => {
                ui::file_error(&file.display().to_string(), &e);
                failed = true;
                None
            }
        })
        .collect();
    (entries, failed)
}

/// Render the type of each JAR, or a single file on its own when its caller
/// named exactly that file.
fn render(files: &[PathBuf], named_only: bool, json: bool) -> Result<bool> {
    let (entries, failed) = collect_entries(files);

    if !entries.is_empty() {
        if json {
            if named_only && entries.len() == 1 {
                println!("{}", to_string_pretty(&entries[0])?);
            } else {
                println!("{}", to_string_pretty(&entries)?);
            }
        } else {
            let pairs: Vec<(String, _)> =
                entries.into_iter().map(|e| (e.file, e.mod_type)).collect();
            match (named_only, pairs.as_slice()) {
                (true, [entry]) => ui::render_type(&entry.1),
                _ => ui::render_type_multi(&pairs),
            }
        }
    }

    Ok(failed)
}

/// Run the `type` command: detect mod/plugin type(s) from JAR files.
///
/// # Errors
///
/// Returns an error if I/O fails or a ZIP archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    let options = args.scan.options();
    let failed = if is_stdin_request(&args.path) {
        match read_stdin()? {
            Stdin::Archive(data) => process_zip_chunks(&data, args.json)?,
            Stdin::Paths(list) => render(&collect_jar_paths(&list, &options)?, false, args.json)?,
        }
    } else {
        let named_only = matches!(args.path.as_slice(), [one] if !one.is_dir());
        render(
            &collect_jar_paths(&args.path, &options)?,
            named_only,
            args.json,
        )?
    };

    if failed {
        return Err(Error::Failed);
    }

    Ok(())
}
