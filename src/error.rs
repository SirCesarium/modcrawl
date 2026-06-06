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
}
