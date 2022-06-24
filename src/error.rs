#[derive(Debug)]
pub enum Error {
    RenderError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
