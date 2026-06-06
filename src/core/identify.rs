use std::io::Read;
use std::path::Path;

use zipcrawl::ZipManager;

use crate::{
    core::detect_mod_type::{detect_mod_type, ModType},
    error::Result,
};

/// Detect the mod/plugin type from a JAR file on disk.
///
/// # Errors
///
/// Returns an error if the file cannot be read or is not a valid ZIP archive.
pub fn identify(path: &Path) -> Result<ModType> {
    let mut mng = ZipManager::new(path)?;
    let entries = mng.entries()?;

    Ok(detect_mod_type(&entries))
}

/// Detect the mod/plugin type from a ZIP archive read via a `Read` impl.
///
/// # Errors
///
/// Returns an error if the data cannot be read or is not a valid ZIP archive.
pub fn identify_reader<R: Read>(reader: &mut R) -> Result<ModType> {
    let mut mng = ZipManager::from_reader(reader)?;
    let entries = mng.entries()?;

    Ok(detect_mod_type(&entries))
}
