use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("AWS error: {0}")]
    Aws(String),
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
