pub mod auth_provider;
pub mod basic;
pub mod aws_iam;
pub mod error;

pub use auth_provider::{AuthProvider, AuthConfig, create_auth_provider};
pub use error::{Error, Result};