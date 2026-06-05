use std::io;
use std::result;

use miette::Diagnostic;
use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    #[diagnostic(code(modcrawl::io))]
    Io(#[from] io::Error),

    #[error("ZipCrawl error")]
    #[diagnostic(code(modcrawl::zipcrawl))]
    ZipCrawl(#[from] zipcrawl::ZipCrawlError),
}
