//! Detect, inspect, and analyze Minecraft mods and plugins from JAR files.
//!
//! The crate exposes loader detection ([`identify`]), metadata parsing
//! ([`read_metadata`]), dependency analysis ([`analyze`]), and side
//! classification ([`side`], [`side_report`]). With the `classfile` feature it
//! can also inspect class files, search the constant pool, find mixin targets,
//! and report duplicate classes ([`find_duplicates`]).
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

pub use core::REGISTRY;
pub use core::dep::types::DepKind;
pub use core::dep::types::VersionRange;
pub use core::dep::{DepEntry, DepReport, JarInJar, analyze, analyze_reader};
pub use core::detect_mod_type::{ForgeModFormat, ModType, PluginType};
pub use core::identify::{identify, identify_reader};
pub use core::metadata::{ModMetadata, read_metadata, read_metadata_reader};
pub use core::registry::{ModHandler, Registry};
pub use core::side::{
    Side, SideReport, classify, side, side_reader, side_report, side_report_reader,
};
pub use error::{Error, Result};

#[cfg(feature = "classfile")]
pub use core::classfile::{
    ClassEntry, ClassScan, DupEntry, GrepMatch, MixinEntry, SkippedClass, find_duplicates,
    find_duplicates_buffers, grep, grep_reader, list_classes, list_classes_reader, mixins,
    mixins_reader, scan_classes, scan_classes_reader, scan_grep, scan_grep_reader, scan_mixins,
    scan_mixins_reader,
};
