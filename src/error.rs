use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no such file or directory: '{0}'")]
    PathNotFound(PathBuf),

    #[error("{0}")]
    RenderError(#[from] tera::Error),

    #[error("invalid {kind} repo: {uri}")]
    InvalidRepo { kind: String, uri: String },

    #[error("invalid alias `{provider}` repo: {alias}")]
    InvalidGitAliasRepo { alias: String, provider: String },

    #[error("{0} `password` is not provided")]
    AuthMissingPassword(String),

    #[error("{0} `username` is not provided")]
    AuthMissingUsername(String),

    #[error("{0}")]
    PromptError(#[from] inquire::error::InquireError),

    #[error("{0}")]
    ArgsError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
