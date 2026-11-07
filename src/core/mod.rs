//! Core analysis primitives: loader detection, metadata, dependencies, and
//! optional class-file inspection.

use std::sync::LazyLock;

use crate::core::registry::Registry;

/// Global registry containing every built-in mod and plugin handler.
pub static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut r = Registry::new();
    loaders::register_all(&mut r);
    r
});

pub mod dep;
pub mod detect_mod_type;
mod embedded;
pub mod identify;
pub mod loaders;
pub mod metadata;
pub mod registry;
pub mod side;

#[cfg(feature = "classfile")]
pub mod classfile;
