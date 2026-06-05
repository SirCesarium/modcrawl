use std::io::{self, BufRead, Read};
use std::path::PathBuf;

use crate::core::identify::{identify, identify_reader};
use crate::error::Result;

const EOCD_MAGIC: &[u8] = b"PK\x05\x06";

#[derive(clap::Args)]
pub struct Args {
    pub file: Vec<PathBuf>,
}

fn load_stdin() -> Result<Vec<u8>> {
    let mut bytes = Vec::new();

    io::stdin().lock().read_to_end(&mut bytes)?;

    Ok(bytes)
}

fn process_zip_chunks(data: &[u8]) {
    // Try as a single ZIP first (covers `cat single.jar`)
    let mut cursor = io::Cursor::new(data);
    if let Ok(mod_type) = identify_reader(&mut cursor) {
        println!("{mod_type}");
        return;
    }

    // Split by EOCD boundaries (covers `cat *.jar`)
    let eocd_positions: Vec<usize> = data
        .windows(4)
        .enumerate()
        .filter(|(_, w)| *w == EOCD_MAGIC)
        .map(|(i, _)| i)
        .collect();

    let mut n = 0;
    let mut prev: usize = 0;

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
            println!("[{n}] {mod_type}");
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
/// Returns an error if I/O fails (reading files, stdin) or if
/// a ZIP archive cannot be read.
pub fn run(args: &Args) -> Result<()> {
    if args.file.is_empty() {
        let mut handle = io::stdin().lock();

        let is_zip = {
            let buf = handle.fill_buf()?;
            buf.starts_with(b"PK\x03\x04")
        };

        if is_zip {
            let mut data = Vec::new();
            handle.read_to_end(&mut data)?;
            process_zip_chunks(&data);
            return Ok(());
        }

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
            let mod_type = identify(&path)?;

            println!("{}: {mod_type}", path.display());
        }
        Ok(())
    } else if args.file.len() == 1 && args.file[0].to_string_lossy() == "-" {
        let data = load_stdin()?;

        process_zip_chunks(&data);

        Ok(())
    } else {
        for file in &args.file {
            let mod_type = identify(file)?;

            println!("{}: {mod_type}", file.display());
        }
        Ok(())
    }
}
