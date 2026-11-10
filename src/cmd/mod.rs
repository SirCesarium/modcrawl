//! Command-line interface and per-command argument handling.

pub mod dep;
pub mod metadata;
pub mod side;
pub mod r#type;

#[cfg(feature = "classfile")]
pub mod classes;

#[cfg(feature = "classfile")]
pub mod grep;

#[cfg(feature = "classfile")]
pub mod mixins;

#[cfg(feature = "classfile")]
pub mod dupes;

use std::collections::HashSet;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, Parser, Subcommand};

use crate::error::{Error, Result};

/// Top-level command-line interface.
#[derive(Parser)]
#[command(name = "modcrawl", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands exposed by the CLI.
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

    /// Classify mods and plugins as client, server, both, or unknown.
    #[command(aliases = &["sd"])]
    Side(side::Args),

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
    /// are being mixed into by each class in the JAR, or detects overlapping
    /// targets across multiple JARs with `--overlaps`.
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

/// Options controlling how directories are scanned for JAR files.
pub(crate) struct ScanOptions {
    /// Recurse into subdirectories.
    pub recursive: bool,
    /// Maximum recursion depth; `None` means unlimited (only meaningful when `recursive` is true).
    pub depth: Option<u32>,
    /// Glob patterns matched against each entry's file name; matching entries are skipped.
    pub exclude: Vec<String>,
}

/// Directory-scanning flags shared by every command that accepts path arguments.
///
/// Flatten this into a command's `Args` with `#[command(flatten)]`.
#[derive(ClapArgs, Default)]
pub struct ScanArgs {
    /// Recurse into subdirectories when a directory is given
    #[arg(short = 'r', long)]
    pub recursive: bool,

    /// Maximum recursion depth (implies --recursive; 1 = direct children only)
    #[arg(long, value_name = "N")]
    pub depth: Option<u32>,

    /// Skip entries whose name matches PATTERN (* matches any characters; repeatable)
    #[arg(long, value_name = "PATTERN")]
    pub exclude: Vec<String>,
}

impl ScanArgs {
    /// Convert to the internal `ScanOptions` used by `collect_jar_paths`.
    pub(crate) fn options(&self) -> ScanOptions {
        ScanOptions {
            recursive: self.recursive || self.depth.is_some(),
            depth: self.depth,
            exclude: self.exclude.clone(),
        }
    }
}

/// Collect the JAR files named by `paths`.
///
/// An argument may be a file or a directory. Without recursion (`options.recursive = false`
/// and `options.max_depth = None`), a directory contributes only its direct children whose
/// extension is `jar` (case-insensitive), sorted by name. With `recursive` or a `max_depth`,
/// subdirectories are walked depth-first. Symbolic links to directories are followed; cycles
/// are detected through a set of canonicalized real paths that have already been visited.
/// Entries whose file name matches any pattern in `options.exclude` are skipped, including
/// entire subdirectories. Any non-directory argument is kept as-is.
///
/// # Errors
///
/// Returns an error if a directory cannot be read.
pub(crate) fn collect_jar_paths(
    paths: &[PathBuf],
    options: &ScanOptions,
) -> io::Result<Vec<PathBuf>> {
    let remaining = if options.recursive || options.depth.is_some() {
        options.depth.map(|d| d.saturating_sub(1))
    } else {
        Some(0)
    };

    let mut jars = Vec::new();
    let mut visited = HashSet::new();

    for path in paths {
        if path.is_dir() {
            collect_jars_from_dir(path, remaining, &options.exclude, &mut visited, &mut jars)?;
        } else {
            jars.push(path.clone());
        }
    }

    Ok(jars)
}

/// Walk `dir`, appending JARs to `out`.
///
/// `remaining` is the number of additional directory levels that may be descended:
/// `Some(0)` = do not enter subdirectories; `None` = unlimited. The function uses
/// `canonicalize` to track real paths in `visited` and silently skips any directory
/// whose canonical path has already been recorded, breaking symlink cycles.
fn collect_jars_from_dir(
    dir: &Path,
    remaining: Option<u32>,
    excludes: &[String],
    visited: &mut HashSet<PathBuf>,
    out: &mut Vec<PathBuf>,
) -> io::Result<()> {
    // Cycle detection: skip directories we have already visited via a different path.
    if let Ok(real) = dir.canonicalize()
        && !visited.insert(real)
    {
        return Ok(());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort();

    let mut jars = Vec::new();
    for entry in &entries {
        let name = entry
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if excludes.iter().any(|pat| glob_match(pat, name)) {
            continue;
        }
        // `is_dir()` follows symlinks; a symlink pointing to a directory is treated as one.
        if entry.is_dir() {
            if remaining != Some(0) {
                collect_jars_from_dir(entry, remaining.map(|d| d - 1), excludes, visited, out)?;
            }
        } else if entry
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
        {
            jars.push(entry.clone());
        }
    }
    out.extend(jars);
    Ok(())
}

/// Match `name` against a glob `pattern` where `*` matches any sequence of characters
/// (including the empty sequence). The match is case-sensitive.
fn glob_match(pattern: &str, name: &str) -> bool {
    let segs: Vec<&str> = pattern.split('*').collect();
    match segs.as_slice() {
        [] => name.is_empty(),
        [only] => *only == name,
        [first, middle @ .., last] => {
            if !name.starts_with(first) || !name.ends_with(last) {
                return false;
            }
            if first.len() + last.len() > name.len() {
                return false;
            }
            let mut rest = &name[first.len()..name.len() - last.len()];
            for seg in middle {
                match rest.find(seg) {
                    Some(pos) => rest = &rest[pos + seg.len()..],
                    None => return false,
                }
            }
            true
        }
    }
}

/// A JAR to process: named on the command line, or piped on stdin.
pub(crate) enum Source {
    Path(PathBuf),
    Archive(Vec<u8>),
}

impl Source {
    /// Name shown in headers and error messages.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Path(path) => path.display().to_string(),
            Self::Archive(_) => String::from("<stdin>"),
        }
    }

    /// The archive's path, if it was named rather than piped.
    fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(path),
            Self::Archive(_) => None,
        }
    }
}

/// What piped stdin carried.
pub(crate) enum Stdin {
    /// A ZIP archive, identified by one of its leading `PK` signatures.
    Archive(Vec<u8>),
    /// Newline-separated paths.
    Paths(Vec<PathBuf>),
}

/// Whether `data` starts with a ZIP local-file, empty-archive, or spanned
/// archive signature.
fn is_zip_archive(data: &[u8]) -> bool {
    data.starts_with(b"PK\x03\x04")
        || data.starts_with(b"PK\x05\x06")
        || data.starts_with(b"PK\x07\x08")
}

/// Whether `paths` asks for stdin: nothing at all, or a lone `-`.
pub(crate) fn is_stdin_request(paths: &[PathBuf]) -> bool {
    paths.is_empty() || (paths.len() == 1 && paths[0] == Path::new("-"))
}

/// Read stdin, failing when it is a terminal so an invocation without
/// arguments reports usage instead of blocking.
///
/// # Errors
///
/// Returns an error if stdin cannot be read, or if it is a terminal.
const MAX_STDIN_SIZE: usize = 256 * 1024 * 1024;

pub(crate) fn read_stdin() -> Result<Stdin> {
    if io::stdin().is_terminal() {
        return Err(Error::Usage(String::from(
            "no input: pass a JAR path or pipe one on stdin",
        )));
    }

    read_stdin_reader(io::stdin().lock())
}

pub(crate) fn read_stdin_reader<R: Read>(reader: R) -> Result<Stdin> {
    let mut data = Vec::new();
    let read = reader
        .take((MAX_STDIN_SIZE as u64) + 1)
        .read_to_end(&mut data)?;
    if read > MAX_STDIN_SIZE {
        return Err(Error::Usage(String::from(
            "stdin exceeds maximum size limit of 256 MB",
        )));
    }

    if is_zip_archive(&data) {
        return Ok(Stdin::Archive(data));
    }

    let list =
        String::from_utf8(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(Stdin::Paths(
        list.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect(),
    ))
}

/// Collect the JARs a command must process.
///
/// A path argument may be a file or a directory, as before. With no argument
/// (or a lone `-`) stdin takes its place: a streamed archive becomes one
/// source, a streamed list of paths is expanded like arguments. The `options`
/// control recursive scanning, depth limits, and exclusion patterns.
///
/// # Errors
///
/// Returns an error if a directory or the piped path list cannot be read, or
/// if stdin is a terminal.
pub(crate) fn collect_sources(paths: &[PathBuf], options: &ScanOptions) -> Result<Vec<Source>> {
    if !is_stdin_request(paths) {
        return Ok(collect_jar_paths(paths, options)?
            .into_iter()
            .map(Source::Path)
            .collect());
    }

    Ok(match read_stdin()? {
        Stdin::Archive(data) => vec![Source::Archive(data)],
        Stdin::Paths(list) => collect_jar_paths(&list, options)?
            .into_iter()
            .map(Source::Path)
            .collect(),
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs as unix_fs;
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

    fn scan(recursive: bool) -> ScanOptions {
        ScanOptions {
            recursive,
            depth: None,
            exclude: vec![],
        }
    }

    #[test]
    fn keeps_explicit_files_in_the_given_order() {
        let dir = TestDir::new("order");
        let b = dir.0.join("b.jar");
        let a = dir.0.join("a.jar");
        fs::write(&b, b"").expect("file is written");
        fs::write(&a, b"").expect("file is written");

        let wanted = [b, a];
        let out = collect_jar_paths(&wanted, &scan(false)).expect("no directory to read");
        assert_eq!(out, wanted);
    }

    #[test]
    fn expands_a_directory_sorted_and_case_insensitive() {
        let dir = TestDir::new("expand");
        for name in ["c.JAR", "a.jar", "b.jar", "notes.txt", "Main.class"] {
            fs::write(dir.0.join(name), b"").expect("file is written");
        }

        let out =
            collect_jar_paths(slice::from_ref(&dir.0), &scan(false)).expect("dir is readable");
        assert_eq!(file_names(&out), ["a.jar", "b.jar", "c.JAR"]);
    }

    #[test]
    fn does_not_recurse_into_subdirectories_by_default() {
        let dir = TestDir::new("no-recurse");
        let nested = dir.0.join("sub");
        fs::create_dir_all(&nested).expect("subdir is created");
        fs::write(nested.join("inner.jar"), b"").expect("file is written");
        fs::write(dir.0.join("outer.jar"), b"").expect("file is written");

        let out =
            collect_jar_paths(slice::from_ref(&dir.0), &scan(false)).expect("dir is readable");
        assert_eq!(out, [dir.0.join("outer.jar")]);
    }

    #[test]
    fn recurses_into_subdirectories_when_enabled() {
        let dir = TestDir::new("recurse-on");
        let nested = dir.0.join("sub");
        fs::create_dir_all(&nested).expect("subdir is created");
        fs::write(nested.join("inner.jar"), b"").expect("file is written");
        fs::write(dir.0.join("outer.jar"), b"").expect("file is written");

        let out = collect_jar_paths(slice::from_ref(&dir.0), &scan(true)).expect("dir is readable");
        // depth-first: subdirectory JARs appear before the containing dir's JARs
        assert!(file_names(&out).contains(&"inner.jar"));
        assert!(file_names(&out).contains(&"outer.jar"));
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn depth_one_does_not_recurse() {
        let dir = TestDir::new("depth-1");
        let nested = dir.0.join("sub");
        fs::create_dir_all(&nested).expect("subdir is created");
        fs::write(nested.join("inner.jar"), b"").expect("file is written");
        fs::write(dir.0.join("outer.jar"), b"").expect("file is written");

        let options = ScanOptions {
            recursive: false,
            depth: Some(1),
            exclude: vec![],
        };
        let out = collect_jar_paths(slice::from_ref(&dir.0), &options).expect("dir is readable");
        assert_eq!(file_names(&out), ["outer.jar"]);
    }

    #[test]
    fn depth_two_reaches_grandchildren() {
        let dir = TestDir::new("depth-2");
        let sub = dir.0.join("sub");
        let subsub = sub.join("deep");
        fs::create_dir_all(&subsub).expect("subdirs are created");
        fs::write(dir.0.join("top.jar"), b"").expect("file is written");
        fs::write(sub.join("mid.jar"), b"").expect("file is written");
        fs::write(subsub.join("deep.jar"), b"").expect("file is written");

        let options = ScanOptions {
            recursive: false,
            depth: Some(2),
            exclude: vec![],
        };
        let out = collect_jar_paths(slice::from_ref(&dir.0), &options).expect("dir is readable");
        let names = file_names(&out);
        assert!(names.contains(&"mid.jar"), "mid.jar should be found");
        assert!(!names.contains(&"deep.jar"), "deep.jar is too deep");
    }

    #[test]
    fn exclude_skips_matching_files() {
        let dir = TestDir::new("exclude-files");
        fs::write(dir.0.join("a.jar"), b"").expect("file is written");
        fs::write(dir.0.join("b.jar"), b"").expect("file is written");
        fs::write(dir.0.join("disabled.jar"), b"").expect("file is written");

        let options = ScanOptions {
            recursive: false,
            depth: None,
            exclude: vec!["disabled*".to_owned()],
        };
        let out = collect_jar_paths(slice::from_ref(&dir.0), &options).expect("dir is readable");
        assert_eq!(file_names(&out), ["a.jar", "b.jar"]);
    }

    #[test]
    fn exclude_skips_matching_directories() {
        let dir = TestDir::new("exclude-dirs");
        let included = dir.0.join("mods");
        let excluded = dir.0.join("disabled");
        fs::create_dir_all(&included).expect("dir is created");
        fs::create_dir_all(&excluded).expect("dir is created");
        fs::write(included.join("good.jar"), b"").expect("file is written");
        fs::write(excluded.join("bad.jar"), b"").expect("file is written");

        let options = ScanOptions {
            recursive: true,
            depth: None,
            exclude: vec!["disabled".to_owned()],
        };
        let out = collect_jar_paths(slice::from_ref(&dir.0), &options).expect("dir is readable");
        assert_eq!(file_names(&out), ["good.jar"]);
    }

    #[test]
    fn symlink_cycle_does_not_loop() {
        let dir = TestDir::new("symlink-cycle");
        let sub = dir.0.join("sub");
        fs::create_dir_all(&sub).expect("subdir is created");
        fs::write(sub.join("mod.jar"), b"").expect("file is written");
        // Create a symlink inside `sub` pointing back to `dir` to form a cycle.
        unix_fs::symlink(&dir.0, sub.join("link_back")).expect("symlink is created");

        let out = collect_jar_paths(slice::from_ref(&dir.0), &scan(true)).expect("does not loop");
        assert_eq!(file_names(&out), ["mod.jar"]);
    }

    #[test]
    fn an_empty_directory_yields_nothing() {
        let dir = TestDir::new("empty");
        let out =
            collect_jar_paths(slice::from_ref(&dir.0), &scan(false)).expect("dir is readable");
        assert_eq!(out, [] as [PathBuf; 0]);
    }

    #[test]
    fn a_missing_path_is_passed_through() {
        let missing = Path::new("/nonexistent/modcrawl-nope.jar").to_path_buf();
        let out = collect_jar_paths(slice::from_ref(&missing), &scan(false))
            .expect("no directory to read");
        assert_eq!(out, [missing]);
    }

    #[test]
    fn glob_match_star_matches_any_suffix() {
        assert!(glob_match("*.jar", "mod.jar"));
        assert!(glob_match("*.jar", ".jar"));
        assert!(!glob_match("*.jar", "mod.zip"));
    }

    #[test]
    fn glob_match_star_matches_any_prefix() {
        assert!(glob_match("mod*", "mod.jar"));
        assert!(glob_match("mod*", "mod"));
        assert!(!glob_match("mod*", "other.jar"));
    }

    #[test]
    fn glob_match_bare_star_matches_everything() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*", ""));
    }

    #[test]
    fn glob_match_no_star_is_exact() {
        assert!(glob_match("disabled", "disabled"));
        assert!(!glob_match("disabled", "disabled.jar"));
    }

    #[test]
    fn recognizes_zip_signatures_on_stdin() {
        assert!(is_zip_archive(b"PK\x03\x04local"));
        assert!(is_zip_archive(b"PK\x05\x06empty"));
        assert!(is_zip_archive(b"PK\x07\x08spanned"));
        assert!(!is_zip_archive(b"a/path/to/mod.jar"));
        assert!(!is_zip_archive(b""));
    }

    #[test]
    fn rejects_oversized_stdin() {
        let large_stream = io::repeat(0).take((MAX_STDIN_SIZE as u64) + 10);
        let result = read_stdin_reader(large_stream);
        assert!(matches!(result, Err(Error::Usage(_))));
    }

    #[test]
    fn command_fails_when_input_jar_fails() {
        let missing = Path::new("/nonexistent/modcrawl-bad.jar").to_path_buf();
        let args = dep::Args {
            path: vec![missing],
            include_jar_in_jar: false,
            json: false,
            scan: ScanArgs::default(),
        };
        assert!(matches!(dep::run(&args), Err(Error::Failed)));
    }
}
