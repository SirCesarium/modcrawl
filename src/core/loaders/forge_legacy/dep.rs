use zipcrawl::ZipManager;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

pub fn extract(mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
    let raw = mng.read_to_string("mcmod.info")?;
    let entries = serde_json::from_str::<Vec<super::metadata::ForgeLegacyModEntry>>(&raw)?;
    let mut deps = Vec::new();

    for entry in entries {
        for dep_name in entry.dependencies {
            deps.push(DepEntry {
                name: dep_name,
                kind: DepKind::Required,
                version_range: VersionRange::parse(None),
            });
        }
    }

    Ok(deps)
}
