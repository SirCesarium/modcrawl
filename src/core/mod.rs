use std::sync::LazyLock;

use crate::core::registry::Registry;

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut r = Registry::new();
    loaders::register_all(&mut r);
    r
});

pub mod dep;
pub mod detect_mod_type;
pub mod identify;
pub mod loaders;
pub mod metadata;
pub mod registry;
