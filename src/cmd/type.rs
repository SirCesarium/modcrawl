use std::io::{self, BufRead, Read};
use std::path::PathBuf;

use crate::cmd::collect_jar_paths;
use crate::core::identify::{identify, identify_reader};
use crate::error::Result;
use crate::ui;

const EOCD_MAGIC: &[u8] = b"PK\x05\x06";

#[derive(clap::Args)]
pub struct Args {
    /// Path of JAR file or directory containing JAR files
    pub path: Vec<PathBuf>,
}

fn process_zip_chunks(data: &[u8]) {
    let mut cursor = io::Cursor::new(data);
    if let Ok(mod_type) = identify_reader(&mut cursor) {
        ui::render_type(&mod_type);
        return;
    }

    let eocd_positions: Vec<usize> = data
        .windows(4)
        .enumerate()
        .filter(|(_, w)| *w == EOCD_MAGIC)
        .map(|(i, _)| i)
        .collect();

    let mut n = 0;
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
            ui::render_type_tagged(n, &mod_type);
        }
        prev = chunk_end;
    }

    if n == 0 {
        eprintln!("No valid ZIPs found in concatenated stream");
    }
}

/// Run the `type` command: detect mod/plugin type(s) from JAR files.
///
/// # Errors
///
/// Returns an error if I/O fails or a ZIP archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    if args.path.is_empty() || (args.path.len() == 1 && args.path[0].to_string_lossy() == "-") {
        let mut handle = io::stdin().lock();
        let buf = handle.fill_buf()?;
        if buf.starts_with(b"PK\x03\x04") {
            let mut data = Vec::new();
            handle.read_to_end(&mut data)?;
            process_zip_chunks(&data);
        } else {
            let mut entries = Vec::new();
            let mut line_buf = String::new();

            loop {
                line_buf.clear();
                if handle.read_line(&mut line_buf)? == 0 {
                    break;
                }

                let line = line_buf.trim().to_owned();
                if line.is_empty() {
                    continue;
                }

                let path = PathBuf::from(line);
                match identify(&path) {
                    Ok(mod_type) => entries.push((path.display().to_string(), mod_type)),
                    Err(e) => ui::file_error(&path.display().to_string(), &e),
                }
            }

            ui::render_type_multi(&entries);
        }
        Ok(())
    } else {
        let single = args.path.len() == 1;
        let entries: Vec<(String, _)> = collect_jar_paths(&args.path)?
            .iter()
            .filter_map(|file| match identify(file) {
                Ok(mod_type) => Some((file.display().to_string(), mod_type)),
                Err(e) => {
                    ui::file_error(&file.display().to_string(), &e);
                    None
                }
            })
            .collect();

        match (single, entries.as_slice()) {
            (true, []) => {}
            (true, [entry]) => ui::render_type(&entry.1),
            _ => ui::render_type_multi(&entries),
        }

        Ok(())
    }
}
