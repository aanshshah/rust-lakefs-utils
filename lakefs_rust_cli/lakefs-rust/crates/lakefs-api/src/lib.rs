pub mod client;
pub mod error;
pub mod models;
pub mod uri;

pub use client::LakeFSClient;
pub use error::{Error, Result};
pub use uri::LakeFSUri;

// Re-export common types
pub use models::{
    Repository, Branch, Commit, ObjectStats,
    DiffResult, MergeResult,
};
