//! Traversal of embedded JARs and the metadata they carry.

use std::io::{self, Cursor, ErrorKind, Read, Seek};

use zipcrawl::ZipManager;

use crate::core::REGISTRY;
use crate::core::dep::find_embedded_jars;
use crate::core::detect_mod_type::ModType;
use crate::core::metadata::read_metadata_string;
use crate::error::{Error, Result};

/// Read the metadata of the first embedded jar that carries its own.
///
/// JarJar-style shells ship the real mod inside `META-INF/jarjar/`, so the
/// outer file only holds a manifest and a packaging descriptor. Returns the
/// detected type alongside the raw metadata, since both callers need the type
/// and only one of them needs the content.
pub(crate) fn read_raw<R: Read + Seek>(
    mng: &mut ZipManager<R>,
) -> Result<Option<(ModType, String)>> {
    let mut found = None;

    visit_embedded(mng, |_path, nested| {
        if found.is_some() {
            return Ok(());
        }

        let Ok(mod_type) = REGISTRY.detect_archive(nested) else {
            return Ok(());
        };

        let Some(file) = REGISTRY.find_metadata_file(&mod_type, &nested.entries()?) else {
            return Ok(());
        };

        let Ok(raw) = read_metadata_string(nested, file) else {
            return Ok(());
        };

        found = Some((mod_type, raw));
        Ok(())
    })?;

    Ok(found)
}

/// Visit every embedded jar the outer archive carries, one at a time.
///
/// JarJar-style shells ship the real mod inside `META-INF/jarjar/`. An entry
/// that cannot be read or that exceeds [`MAX_EMBEDDED_JAR_SIZE`] is skipped so
/// one broken sibling does not abort the caller's work. Only one level is
/// traversed, matching how metadata discovery sees a shell.
///
/// # Errors
///
/// Returns an error when the outer archive cannot be listed, or when `visit`
/// returns one.
pub(crate) fn visit_embedded<R, F>(mng: &mut ZipManager<R>, mut visit: F) -> Result<()>
where
    R: Read + Seek,
    F: FnMut(&str, &mut ZipManager<Cursor<Vec<u8>>>) -> Result<()>,
{
    for jar in find_embedded_jars(&mng.entries()?) {
        let Ok(mut nested) = read_entry(mng, &jar.path) else {
            continue;
        };

        visit(&jar.path, &mut nested)?;
    }

    Ok(())
}

const MAX_EMBEDDED_JAR_SIZE: usize = 128 * 1024 * 1024;

fn read_entry<R: Read + Seek>(
    mng: &mut ZipManager<R>,
    name: &str,
) -> Result<ZipManager<Cursor<Vec<u8>>>> {
    let mut data = Vec::new();
    let file = mng.open_file(name)?;
    let read = file
        .take((MAX_EMBEDDED_JAR_SIZE as u64) + 1)
        .read_to_end(&mut data)
        .map_err(Error::Io)?;
    if read > MAX_EMBEDDED_JAR_SIZE {
        return Err(Error::Io(io::Error::new(
            ErrorKind::InvalidData,
            "embedded archive entry exceeds size limit",
        )));
    }

    Ok(ZipManager::from_reader(&mut Cursor::new(data))?)
}
