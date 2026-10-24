use std::io::{Cursor, Read, Seek};

use zipcrawl::ZipManager;

use crate::core::REGISTRY;
use crate::core::dep::find_embedded_jars;
use crate::core::detect_mod_type::ModType;
use crate::error::Result;

/// Read the metadata of the first embedded jar that carries its own.
///
/// JarJar-style shells ship the real mod inside `META-INF/jarjar/`, so the
/// outer file only holds a manifest and a packaging descriptor. Returns the
/// detected type alongside the raw metadata, since both callers need the type
/// and only one of them needs the content.
pub(crate) fn read_raw<R: Read + Seek>(
    mng: &mut ZipManager<R>,
) -> Result<Option<(ModType, String)>> {
    for jar in find_embedded_jars(&mng.entries()?) {
        let Ok(mut nested) = read_entry(mng, &jar.path) else {
            continue;
        };

        let mod_type = REGISTRY.detect(&nested.entries()?);

        let Some(file) = REGISTRY.metadata_file_path(&mod_type) else {
            continue;
        };

        let raw = nested.read_to_string(file)?;
        return Ok(Some((mod_type, raw)));
    }

    Ok(None)
}

fn read_entry<R: Read + Seek>(
    mng: &mut ZipManager<R>,
    name: &str,
) -> Result<ZipManager<Cursor<Vec<u8>>>> {
    let mut data = Vec::new();
    mng.open_file(name)?.read_to_end(&mut data)?;

    Ok(ZipManager::from_reader(&mut Cursor::new(data))?)
}
