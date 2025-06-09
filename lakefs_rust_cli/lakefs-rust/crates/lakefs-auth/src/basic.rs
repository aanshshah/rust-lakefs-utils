use crate::{auth_provider::AuthProvider, error::Result};
use async_trait::async_trait;
use base64::Engine;

pub struct BasicAuth {
    access_key_id: String,
    secret_access_key: String,
}

impl BasicAuth {
    pub fn new(access_key_id: String, secret_access_key: String) -> Self {
        Self {
            access_key_id,
            secret_access_key,
        }
    }
}

#[async_trait]
impl AuthProvider for BasicAuth {
    async fn get_auth_header(&self) -> Result<String> {
        let credentials = format!("{}:{}", self.access_key_id, self.secret_access_key);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        Ok(format!("Basic {}", encoded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_auth_header() {
        let auth = BasicAuth::new("user".to_string(), "pass".to_string());
        let header = auth.get_auth_header().await.unwrap();
        
        // "user:pass" base64 encoded is "dXNlcjpwYXNz"
        assert_eq!(header, "Basic dXNlcjpwYXNz");
    }

    #[tokio::test]
    async fn test_basic_auth_special_chars() {
        let auth = BasicAuth::new("user@example.com".to_string(), "p@$$w0rd!".to_string());
        let header = auth.get_auth_header().await.unwrap();
        
        assert!(header.starts_with("Basic "));
        
        // Decode and verify
        let encoded = header.strip_prefix("Basic ").unwrap();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        
        assert_eq!(decoded_str, "user@example.com:p@$$w0rd!");
    }
}
