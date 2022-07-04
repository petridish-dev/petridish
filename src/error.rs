use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not found '{path}'")]
    PathNotFound {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },

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

    #[error("ParseError")]
    ParseError(#[from] toml::de::Error),

    #[error("exists '{0}' in output dir, cannot overwrite it, or you can add flag `-f` or `-s`")]
    CannotOverwriteContent(PathBuf),

    #[error("git error")]
    GitError(#[from] git2::Error),

    #[error("unknown git ref '{0}'")]
    InvalidGitRef(String),

    #[error("not found repo '{0}' in cache")]
    RepoNotFoundInCache(String),
}

pub type Result<T> = std::result::Result<T, Error>;
