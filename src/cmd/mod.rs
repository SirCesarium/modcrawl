pub mod dep;
pub mod metadata;
pub mod r#type;

#[cfg(feature = "classfile")]
pub mod classes;

#[cfg(feature = "classfile")]
pub mod grep;

#[cfg(feature = "classfile")]
pub mod mixins;

#[cfg(feature = "classfile")]
pub mod dupes;

use std::{fs, io, path::PathBuf};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "modcrawl", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze & crawl `Minecraft` mod dependencies.
    ///
    /// Finds hidden dependencies in mod metadata.
    ///
    /// By default, ignores Jar-In-Jar dependencies, use `--include-jar-in-jar` to include them.
    #[command(aliases = &["d", "deps", "dependencies"])]
    Dep(dep::Args),

    /// Gets `mod.jar` metadata.
    #[command(aliases = &["meta", "md"])]
    Metadata(metadata::Args),

    /// Get mod type. (eg. `NeoForge` mod, `Fabric` mod, ...)
    #[command(name = "type", aliases = &["t", "ty"])]
    Type(r#type::Args),

    /// List all `.class` files inside a JAR with Java version and access flags.
    #[cfg(feature = "classfile")]
    #[command(aliases = &["c", "cls"])]
    Classes(classes::Args),

    /// Search strings in the constant pool of all `.class` files inside a JAR.
    ///
    /// Useful for finding references to classes, methods, fields, or annotations.
    #[cfg(feature = "classfile")]
    #[command(aliases = &["g", "search"])]
    Grep(grep::Args),

    /// Extract `@Mixin` targets from class-level annotations.
    ///
    /// Looks for `SpongePowered` `@Mixin` annotations to find which classes
    /// are being mixed into by each class in the JAR.
    #[cfg(feature = "classfile")]
    #[command(aliases = &["m", "mixin"])]
    Mixins(mixins::Args),

    /// Find duplicate `.class` entries across multiple JARs.
    ///
    /// Useful for detecting classpath conflicts between mods.
    #[cfg(feature = "classfile")]
    #[command(aliases = &["dp", "duplicate"])]
    Dupes(dupes::Args),
}

/// Collects jar paths from the given files and directories.
///
/// The input may mix files and directories. For a directory, only its **direct**
/// child files with `.jar` suffix are collected. Other paths are kept as-is.
///
/// # Errors
///
/// Return an error if a directory cannot be read.
pub fn collect_jar_paths(paths: &Vec<PathBuf>) -> io::Result<Vec<PathBuf>> {
    let mut ret = Vec::new();
    for path in paths {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let path = entry?.path();
                if path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
                {
                    ret.push(path);
                }
            }
        } else {
            ret.push(path.clone());
        }
    }

    Ok(ret)
}
