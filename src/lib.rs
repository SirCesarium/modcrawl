#[cfg(feature = "cli")]
pub mod cmd;
pub mod core;
pub mod error;
pub mod ffi;
#[cfg(feature = "cli")]
pub(crate) mod ui;

// ── Re-exports ──────────────────────────────────────────────────────────────

pub use core::dep::types::DepKind;
pub use core::dep::types::VersionRange;
pub use core::dep::{DepEntry, DepReport, JarInJar, analyze, analyze_reader};
pub use core::detect_mod_type::ModType;
pub use core::identify::{identify, identify_reader};
pub use core::metadata::{ModMetadata, read_metadata, read_metadata_reader};
pub use error::{Error, Result};

#[cfg(feature = "classfile")]
pub use core::classfile::{ClassEntry, DupEntry, GrepMatch, MixinEntry};
