//! Error types shared by the library and the CLI.

use std::error::Error as StdError;
use std::io;
use std::result;

#[cfg(feature = "cli")]
use miette::Diagnostic;
use serde_json::Error as JsonError;
use serde_saphyr::Error as YamlError;
use thiserror::Error;
use zipcrawl::ZipCrawlError;

use toml::de;

/// Convenience result type used throughout the crate.
pub type Result<T> = result::Result<T, Error>;

/// Deepest cause in a `zipcrawl` error's source chain.
///
/// `IoError` wraps an `io::Error`, so the leaf (`No such file or directory`)
/// carries the reason and the wrapper only carries the path.
fn zipcrawl_reason(err: &ZipCrawlError) -> String {
    let mut cause: &dyn StdError = err;
    while let Some(source) = cause.source() {
        cause = source;
    }

    let reason = cause.to_string();
    if reason.is_empty() {
        err.to_string()
    } else {
        reason
    }
}

/// Errors produced while reading or parsing mod and archive data.
#[derive(Error, Debug)]
#[cfg_attr(feature = "cli", derive(Diagnostic))]
pub enum Error {
    /// An underlying I/O operation failed.
    #[error("I/O error: {0}")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::io)))]
    Io(#[from] io::Error),

    /// Reading the ZIP/JAR archive failed.
    #[error("{}", zipcrawl_reason(.0))]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::zipcrawl)))]
    ZipCrawl(#[from] ZipCrawlError),

    /// JSON metadata could not be parsed.
    #[error("JSON error: {0}")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::json)))]
    Json(#[from] JsonError),

    /// TOML metadata could not be parsed.
    #[error("TOML error: {0}")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::toml)))]
    Toml(#[from] de::Error),

    /// YAML metadata could not be parsed.
    #[error("YAML error: {0}")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::yaml)))]
    Yaml(#[from] YamlError),

    /// The archive does not contain metadata for a supported loader.
    #[error("Unsupported metadata format for {0}")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::unsupported_metadata)))]
    UnsupportedMetadata(String),

    /// The command was invoked with invalid arguments.
    #[error("{0}")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::usage)))]
    Usage(String),

    /// A class file could not be read or interpreted.
    #[cfg(feature = "classfile")]
    #[error("Class file error: {0}")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::classfile)))]
    ClassFile(String),

    /// A class-file structure could not be parsed.
    #[cfg(feature = "classfile")]
    #[error("Parse error: {0}")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::parse)))]
    Parse(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn error_display_io() {
        let e = Error::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        assert!(e.to_string().contains("file not found"));
        assert!(e.to_string().contains("I/O error"));
    }

    #[test]
    fn error_display_unsupported_metadata() {
        let e = Error::UnsupportedMetadata("Foo.jar".to_owned());
        assert_eq!(e.to_string(), "Unsupported metadata format for Foo.jar");
    }

    #[test]
    fn error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let e: Error = io_err.into();
        assert!(matches!(e, Error::Io(_)));
    }

    #[test]
    fn error_usage_shows_only_its_message() {
        let e = Error::Usage(String::from("no input: pass a JAR path"));
        assert_eq!(e.to_string(), "no input: pass a JAR path");
    }

    #[cfg(feature = "cli")]
    #[test]
    fn error_usage_code() {
        use miette::Diagnostic;
        let e = Error::Usage(String::from("no input"));
        assert_eq!(e.code().unwrap().to_string(), "modcrawl::usage");
    }

    #[cfg(feature = "cli")]
    #[test]
    fn error_unsupported_metadata_code() {
        use miette::Diagnostic;
        let e = Error::UnsupportedMetadata("test".to_owned());
        assert_eq!(
            e.code().unwrap().to_string(),
            "modcrawl::unsupported_metadata"
        );
    }
}
