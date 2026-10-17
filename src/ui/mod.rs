//! The terminal presentation layer for the CLI subcommands.
//!
//! Every human-readable output path renders through here so the whole tool
//! reads as one coherent design: borderless tables, a shared color palette,
//! and explicit "no results" notes. `--json` paths bypass this module and
//! serialize the core types directly.

pub(crate) mod classes;
pub(crate) mod deps;
pub(crate) mod dupes;
pub(crate) mod grep;
pub(crate) mod lines;
pub(crate) mod metadata;
pub(crate) mod mixins;
pub(crate) mod style;
pub(crate) mod table;
pub(crate) mod r#type;

pub(crate) use classes::render as render_classes;
pub(crate) use deps::render as render_deps;
pub(crate) use dupes::render as render_dupes;
pub(crate) use grep::render as render_grep;
pub(crate) use lines::{file_error, file_header, no_results};
pub(crate) use metadata::render as render_metadata;
pub(crate) use mixins::render as render_mixins;
pub(crate) use r#type::{render as render_type, render_multi as render_type_multi};
