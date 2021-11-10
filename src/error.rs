//! # Errors

use miette::Diagnostic;
use serde_yaml::Error as YamlError;
use thiserror::Error;

/// Petridish error type
#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("repo: {name}, error: {error}")]
    Repo { name: String, error: String },

    #[error("parse error: {0}")]
    ParseError(#[from] YamlError),

    #[error("validate field '{field}' failed: {error}")]
    ValidateError { field: String, error: String },

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
