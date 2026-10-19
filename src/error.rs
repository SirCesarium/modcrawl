use std::error::Error as StdError;
use std::io;
use std::result;

use miette::Diagnostic;
use serde_json::Error as JsonError;
use serde_saphyr::Error as YamlError;
use thiserror::Error;
use zipcrawl::ZipCrawlError;

use toml::de;

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

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    #[diagnostic(code(modcrawl::io))]
    Io(#[from] io::Error),

    #[error("{}", zipcrawl_reason(.0))]
    #[diagnostic(code(modcrawl::zipcrawl))]
    ZipCrawl(#[from] ZipCrawlError),

    #[error("JSON error: {0}")]
    #[diagnostic(code(modcrawl::json))]
    Json(#[from] JsonError),

    #[error("TOML error: {0}")]
    #[diagnostic(code(modcrawl::toml))]
    Toml(#[from] de::Error),

    #[error("YAML error: {0}")]
    #[diagnostic(code(modcrawl::yaml))]
    Yaml(#[from] YamlError),

    #[error("Unsupported metadata format for {0}")]
    #[diagnostic(code(modcrawl::unsupported_metadata))]
    UnsupportedMetadata(String),

    #[cfg(feature = "classfile")]
    #[error("Class file error: {0}")]
    #[diagnostic(code(modcrawl::classfile))]
    ClassFile(String),

    #[cfg(feature = "classfile")]
    #[error("Parse error: {0}")]
    #[diagnostic(code(modcrawl::parse))]
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
    fn error_unsupported_metadata_code() {
        use miette::Diagnostic;
        let e = Error::UnsupportedMetadata("test".to_owned());
        assert_eq!(
            e.code().unwrap().to_string(),
            "modcrawl::unsupported_metadata"
        );
    }
}
