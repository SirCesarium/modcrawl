use zipcrawl::ZipManager;

use super::super::super::dep::types::DepEntry;
use crate::error::Result;

#[allow(clippy::unnecessary_wraps)]
pub fn extract(_: &mut ZipManager) -> Result<Vec<DepEntry>> {
    Ok(Vec::new())
}
