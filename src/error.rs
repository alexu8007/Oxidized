use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>; 