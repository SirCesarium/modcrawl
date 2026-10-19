//! Terminal presentation layer for the CLI subcommands. `--json` paths bypass
//! it and serialize the core types directly.

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
pub(crate) use r#type::{
    render as render_type, render_multi as render_type_multi, render_tagged as render_type_tagged,
};
