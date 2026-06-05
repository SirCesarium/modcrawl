pub mod dep;
pub mod metadata;
pub mod r#type;

use clap::Parser;

#[derive(Parser)]
#[command(name = "modcrawl", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Analyze & crawl `Minecraft` mod dependencies.
    /// 
    /// Finds hidden dependencies in mod metadata.
    /// 
    /// By default, ignores Jar-In-Jar dependencies, use `-j` to include them.
    #[command(aliases = &["d", "deps", "dependencies"])]
    Dep(dep::Args),

    /// Gets `mod.jar` metadata.
    #[command(aliases = &["meta", "md"])]
    Metadata(metadata::Args),

    /// Get mod type. (eg. `NeoForge` mod, `Fabric` mod, ...)
    #[command(name = "type", aliases = &["t", "ty"])]
    Type(r#type::Args),
}
