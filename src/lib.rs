//! Detect, inspect, and analyze Minecraft mods and plugins from JAR files.
//!
//! The crate exposes loader detection ([`identify`]), metadata parsing
//! ([`read_metadata`]), and dependency analysis ([`analyze`]). With the
//! `classfile` feature it can also inspect class files, search the constant
//! pool, find mixin targets, and report duplicate classes.
//!
//! Feature flags:
//! - `cli` enables the command-line interface in `cmd`.
//! - `classfile` enables class-file analysis in `core::classfile`.
//! - `wasm` enables the WebAssembly bindings in `wasm`.

#[cfg(feature = "cli")]
pub mod cmd;
pub mod core;
pub mod error;
#[cfg(feature = "cli")]
pub(crate) mod ui;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;

pub use core::dep::types::DepKind;
pub use core::dep::types::VersionRange;
pub use core::dep::{DepEntry, DepReport, JarInJar, analyze, analyze_reader};
pub use core::detect_mod_type::ModType;
pub use core::identify::{identify, identify_reader};
pub use core::metadata::{ModMetadata, read_metadata, read_metadata_reader};
pub use core::side::{Side, classify};
pub use error::{Error, Result};

#[cfg(feature = "classfile")]
pub use core::classfile::{ClassEntry, DupEntry, GrepMatch, MixinEntry};
