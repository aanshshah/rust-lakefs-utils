use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("API error: {0}")]
    Api(#[from] lakefs_api::Error),
    
    #[error("Index error: {0}")]
    Index(String),
    
    #[error("Sync error: {0}")]
    Sync(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, Error>;