//! # Errors

use miette::Diagnostic;
use thiserror::Error;

/// Petridish error type
#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("[repo_error] repo: {name}, error: {error}")]
    Repo { name: String, error: String },
}

pub type Result<T> = std::result::Result<T, Error>;
