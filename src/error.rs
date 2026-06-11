use std::io;
use std::result;

use miette::Diagnostic;
use thiserror::Error;

use toml::de;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    #[diagnostic(code(modcrawl::io))]
    Io(#[from] io::Error),

    #[error("ZipCrawl error")]
    #[diagnostic(code(modcrawl::zipcrawl))]
    ZipCrawl(#[from] zipcrawl::ZipCrawlError),

    #[error("JSON error: {0}")]
    #[diagnostic(code(modcrawl::json))]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    #[diagnostic(code(modcrawl::toml))]
    Toml(#[from] de::Error),

    #[error("YAML error: {0}")]
    #[diagnostic(code(modcrawl::yaml))]
    Yaml(#[from] serde_saphyr::Error),

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
