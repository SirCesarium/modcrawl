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

/// Collect the JAR files named by `paths`.
///
/// An argument may be a file or a directory; a directory contributes its direct
/// children whose extension is `jar` (case-insensitive), sorted by name. Any
/// other argument is kept as-is, so an explicit non-JAR path still reaches the
/// caller and gets reported by it.
///
/// # Errors
///
/// Returns an error if a directory cannot be read.
pub(crate) fn collect_jar_paths(paths: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    let mut jars = Vec::new();

    for path in paths {
        if path.is_dir() {
            let mut dir_jars: Vec<PathBuf> = fs::read_dir(path)?
                .map(|entry| entry.map(|e| e.path()))
                .collect::<io::Result<Vec<_>>>()?
                .into_iter()
                .filter(|p| {
                    p.extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
                })
                .collect();
            dir_jars.sort();
            jars.extend(dir_jars);
        } else {
            jars.push(path.clone());
        }
    }

    Ok(jars)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::slice;

    use super::*;

    /// A scratch directory that removes itself when the test ends.
    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let dir = env::temp_dir().join(format!("modcrawl-cmd-{name}"));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("temp dir is created");
            Self(dir)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn file_names(jars: &[PathBuf]) -> Vec<&str> {
        jars.iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .collect()
    }

    #[test]
    fn keeps_explicit_files_in_the_given_order() {
        let dir = TestDir::new("order");
        let b = dir.0.join("b.jar");
        let a = dir.0.join("a.jar");
        fs::write(&b, b"").expect("file is written");
        fs::write(&a, b"").expect("file is written");

        let wanted = [b, a];
        let out = collect_jar_paths(&wanted).expect("no directory to read");
        assert_eq!(out, wanted);
    }

    #[test]
    fn expands_a_directory_sorted_and_case_insensitive() {
        let dir = TestDir::new("expand");
        for name in ["c.JAR", "a.jar", "b.jar", "notes.txt", "Main.class"] {
            fs::write(dir.0.join(name), b"").expect("file is written");
        }

        let out = collect_jar_paths(slice::from_ref(&dir.0)).expect("dir is readable");
        assert_eq!(file_names(&out), ["a.jar", "b.jar", "c.JAR"]);
    }

    #[test]
    fn does_not_recurse_into_subdirectories() {
        let dir = TestDir::new("recurse");
        let nested = dir.0.join("sub");
        fs::create_dir_all(&nested).expect("subdir is created");
        fs::write(nested.join("inner.jar"), b"").expect("file is written");
        fs::write(dir.0.join("outer.jar"), b"").expect("file is written");

        let out = collect_jar_paths(slice::from_ref(&dir.0)).expect("dir is readable");
        assert_eq!(out, [dir.0.join("outer.jar")]);
    }

    #[test]
    fn an_empty_directory_yields_nothing() {
        let dir = TestDir::new("empty");
        let out = collect_jar_paths(slice::from_ref(&dir.0)).expect("dir is readable");
        assert_eq!(out, [] as [PathBuf; 0]);
    }

    #[test]
    fn a_missing_path_is_passed_through() {
        let missing = Path::new("/nonexistent/modcrawl-nope.jar").to_path_buf();
        let out = collect_jar_paths(slice::from_ref(&missing)).expect("no directory to read");
        assert_eq!(out, [missing]);
    }
}
