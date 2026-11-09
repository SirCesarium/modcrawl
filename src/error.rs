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

/// Render a `zipcrawl` error, keeping the archive path and the underlying
/// reason together.
///
/// `IoError` stores the path on the wrapper and the reason on its source, so
/// rendering either one alone loses half the diagnostic. Other variants fall
/// back to their full source chain.
fn zipcrawl_reason(err: &ZipCrawlError) -> String {
    if let ZipCrawlError::IoError { path, source } = err {
        return format!("{path}: {source}");
    }

    let mut messages: Vec<String> = Vec::new();
    let mut cause: &dyn StdError = err;

    loop {
        let message = cause.to_string();
        if !message.is_empty() && messages.last() != Some(&message) {
            messages.push(message);
        }

        match cause.source() {
            Some(source) => cause = source,
            None => break,
        }
    }

    messages.join(": ")
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

    /// One or more inputs failed to process.
    #[error("Failed to process one or more inputs")]
    #[cfg_attr(feature = "cli", diagnostic(code(modcrawl::failed)))]
    Failed,

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
    fn error_display_zipcrawl_keeps_the_archive_path() {
        let e = Error::ZipCrawl(ZipCrawlError::IoError {
            path: String::from("missing.jar"),
            source: io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
        });

        let message = e.to_string();
        assert!(message.contains("missing.jar"));
        assert!(message.contains("No such file or directory"));
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

    #[test]
    fn error_failed_display() {
        let e = Error::Failed;
        assert_eq!(e.to_string(), "Failed to process one or more inputs");
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
