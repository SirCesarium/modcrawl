use zipcrawl::ZipManager;

use serde_json::Value;

use super::super::super::dep::types::{DepEntry, DepKind, VersionRange};
use crate::error::Result;

fn ver(value: Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s),
        Value::Array(arr) => arr.into_iter().find_map(|v| v.as_str().map(String::from)),
        _ => None,
    }
}

pub fn extract(mng: &mut ZipManager) -> Result<Vec<DepEntry>> {
    let raw = mng.read_to_string("fabric.mod.json")?;
    let meta = serde_json::from_str::<super::metadata::FabricModMetadata>(&raw)?;
    let mut deps = Vec::new();

    for (name, range) in meta.depends {
        deps.push(DepEntry::new(
            name,
            DepKind::Required,
            VersionRange::parse(ver(range)),
        ));
    }
    for (name, range) in meta.recommends {
        deps.push(DepEntry::new(
            name,
            DepKind::Recommended,
            VersionRange::parse(ver(range)),
        ));
    }
    for (name, range) in meta.suggests {
        deps.push(DepEntry::new(
            name,
            DepKind::Suggested,
            VersionRange::parse(ver(range)),
        ));
    }

    Ok(deps)
}
