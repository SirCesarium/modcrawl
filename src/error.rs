use miette::Diagnostic;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    // ...
}
