#[cfg(feature = "cli")]
pub mod cmd;
pub mod core;
pub mod error;
pub mod ffi;

// ── Re-exports ──────────────────────────────────────────────────────────────

pub use core::dep::types::DepKind;
pub use core::dep::types::VersionRange;
pub use core::dep::{analyze, analyze_reader, DepEntry, DepReport, JarInJar};
pub use core::detect_mod_type::ModType;
pub use core::identify::{identify, identify_reader};
pub use core::metadata::{read_metadata, read_metadata_reader, ModMetadata};
pub use error::{Error, Result};

#[cfg(feature = "classfile")]
pub use core::classfile::{ClassEntry, DupEntry, GrepMatch, MixinEntry};
