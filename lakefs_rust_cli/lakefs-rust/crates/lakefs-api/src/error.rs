use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
    
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
