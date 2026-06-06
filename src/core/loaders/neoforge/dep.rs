use zipcrawl::ZipManager;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use super::super::forge_modern::metadata::ModsTomlMetadata;
use crate::error::Result;

pub fn extract(mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
    let raw = mng.read_to_string("META-INF/neoforge.mods.toml")?;
    let meta = toml::from_str::<ModsTomlMetadata>(&raw)?;
    let mut deps = Vec::new();

    for deps_vec in meta.dependencies.values() {
        for dep in deps_vec {
            let kind = match dep.dep_type.as_deref() {
                Some("optional") => DepKind::Optional,
                Some("incompatible") => DepKind::Incompatible,
                Some("discouraged") => DepKind::Discouraged,
                _ => DepKind::Required,
            };
            if kind.is_excluded() {
                continue;
            }
            deps.push(DepEntry {
                name: dep.mod_id.clone(),
                kind,
                version_range: VersionRange::parse(dep.version_range.clone()),
            });
        }
    }

    Ok(deps)
}
