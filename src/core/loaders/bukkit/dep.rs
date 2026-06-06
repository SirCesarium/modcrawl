use zipcrawl::ZipManager;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

pub fn extract(mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
    let raw = mng.read_to_string("plugin.yml")?;
    let meta = serde_saphyr::from_str::<super::metadata::BukkitPluginMetadata>(&raw)?;
    let mut deps = Vec::new();

    for name in meta.depend {
        deps.push(DepEntry {
            name,
            kind: DepKind::Required,
            version_range: VersionRange::parse(None),
        });
    }
    for name in meta.softdepend {
        deps.push(DepEntry {
            name,
            kind: DepKind::Optional,
            version_range: VersionRange::parse(None),
        });
    }
    for name in meta.loadbefore {
        deps.push(DepEntry {
            name,
            kind: DepKind::LoadBefore,
            version_range: VersionRange::parse(None),
        });
    }

    Ok(deps)
}
