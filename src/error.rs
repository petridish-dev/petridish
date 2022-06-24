#[derive(Debug, PartialEq)]
pub enum Error {
    RenderError(String),

    InvalidRepo(String),

    ArgsError(String),

    ConfigNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
