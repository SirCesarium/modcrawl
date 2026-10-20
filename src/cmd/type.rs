use std::io;
use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::cmd::{Stdin, collect_jar_paths, is_stdin_request, read_stdin};
use crate::core::identify::{identify, identify_reader};
use crate::error::Result;
use crate::ui;

const EOCD_MAGIC: &[u8] = b"PK\x05\x06";

#[derive(ClapArgs)]
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

/// Render the type of each JAR, or a single file on its own when its caller
/// named exactly that file.
fn render(files: &[PathBuf], named_only: bool) {
    let entries: Vec<(String, _)> = files
        .iter()
        .filter_map(|file| match identify(file) {
            Ok(mod_type) => Some((file.display().to_string(), mod_type)),
            Err(e) => {
                ui::file_error(&file.display().to_string(), &e);
                None
            }
        })
        .collect();

    match (named_only, entries.as_slice()) {
        (true, [entry]) => ui::render_type(&entry.1),
        _ => ui::render_type_multi(&entries),
    }
}

/// Run the `type` command: detect mod/plugin type(s) from JAR files.
///
/// # Errors
///
/// Returns an error if I/O fails or a ZIP archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    if is_stdin_request(&args.path) {
        match read_stdin()? {
            Stdin::Archive(data) => process_zip_chunks(&data),
            Stdin::Paths(list) => render(&collect_jar_paths(&list)?, false),
        }
        return Ok(());
    }

    // A directory is always rendered as a table: its caller never named the
    // file, so the FILE column is the only thing identifying it.
    let named_only = matches!(args.path.as_slice(), [one] if !one.is_dir());

    render(&collect_jar_paths(&args.path)?, named_only);

    Ok(())
}
