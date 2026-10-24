use std::io::{Read, Seek};
use std::path::Path;

use zipcrawl::ZipManager;

use crate::core::REGISTRY;
use crate::core::detect_mod_type::ModType;
use crate::core::embedded;
use crate::error::Result;

/// Detect the mod/plugin type from a JAR file on disk.
///
/// # Errors
///
/// Returns an error if the file cannot be read or is not a valid ZIP archive.
pub fn identify(path: &Path) -> Result<ModType> {
    let mut mng = ZipManager::new(path)?;
    identify_impl(&mut mng)
}

/// Detect the mod/plugin type from a ZIP archive read via a `Read` impl.
///
/// # Errors
///
/// Returns an error if the data cannot be read or is not a valid ZIP archive.
pub fn identify_reader<R: Read>(reader: &mut R) -> Result<ModType> {
    let mut mng = ZipManager::from_reader(reader)?;
    identify_impl(&mut mng)
}

fn identify_impl<R: Read + Seek>(mng: &mut ZipManager<R>) -> Result<ModType> {
    let entries = mng.entries()?;
    let mod_type = REGISTRY.detect(&entries);

    if mod_type != ModType::Unknown {
        return Ok(mod_type);
    }

    let Some((mod_type, _)) = embedded::read_raw(mng)? else {
        return Ok(ModType::Unknown);
    };

    Ok(mod_type)
}
