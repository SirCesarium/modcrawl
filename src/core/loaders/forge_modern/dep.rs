use zipcrawl::ZipManager;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

pub fn extract(mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
    let raw = mng.read_to_string("META-INF/mods.toml")?;
    let meta = toml::from_str::<super::metadata::ModsTomlMetadata>(&raw)?;
    let mut deps = Vec::new();

    for deps_vec in meta.dependencies.values() {
        for dep in deps_vec {
            let kind = forge_dep_kind(dep.dep_type.as_deref());
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

fn forge_dep_kind(dep_type: Option<&str>) -> DepKind {
    match dep_type {
        Some("optional") => DepKind::Optional,
        Some("incompatible") => DepKind::Incompatible,
        Some("discouraged") => DepKind::Discouraged,
        _ => DepKind::Required,
    }
}
