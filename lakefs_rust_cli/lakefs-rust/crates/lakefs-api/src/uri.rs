use std::str::FromStr;
use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct LakeFSUri {
    pub repository: String,
    pub reference: String,
    pub path: Option<String>,
}

impl LakeFSUri {
    pub fn new(repository: impl Into<String>, reference: impl Into<String>) -> Self {
        Self {
            repository: repository.into(),
            reference: reference.into(),
            path: None,
        }
    }
    
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    
    pub fn to_string(&self) -> String {
        match &self.path {
            Some(path) => format!("lakefs://{}/{}/{}", self.repository, self.reference, path),
            None => format!("lakefs://{}/{}", self.repository, self.reference),
        }
    }
}

impl FromStr for LakeFSUri {
    type Err = Error;
    
    fn from_str(s: &str) -> Result<Self> {
        if !s.starts_with("lakefs://") {
            return Err(Error::InvalidUri("URI must start with lakefs://".into()));
        }
        
        let path = &s["lakefs://".len()..];
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        
        match parts.len() {
            0 | 1 => Err(Error::InvalidUri("Missing repository and reference".into())),
            2 => Ok(Self {
                repository: parts[0].to_string(),
                reference: parts[1].to_string(),
                path: None,
            }),
            3 => Ok(Self {
                repository: parts[0].to_string(),
                reference: parts[1].to_string(),
                path: Some(parts[2].to_string()),
            }),
            _ => unreachable!(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_from_str_with_path() {
        let uri = LakeFSUri::from_str("lakefs://my-repo/my-branch/path/to/file").unwrap();
        assert_eq!(uri.repository, "my-repo");
        assert_eq!(uri.reference, "my-branch");
        assert_eq!(uri.path, Some("path/to/file".to_string()));
    }

    #[test]
    fn test_uri_from_str_without_path() {
        let uri = LakeFSUri::from_str("lakefs://my-repo/my-branch").unwrap();
        assert_eq!(uri.repository, "my-repo");
        assert_eq!(uri.reference, "my-branch");
        assert_eq!(uri.path, None);
    }

    #[test]
    fn test_uri_from_str_invalid() {
        assert!(LakeFSUri::from_str("invalid://uri").is_err());
        assert!(LakeFSUri::from_str("lakefs://").is_err());
        assert!(LakeFSUri::from_str("lakefs://repo").is_err());
    }

    #[test]
    fn test_uri_to_string() {
        let uri = LakeFSUri::new("repo", "branch").with_path("path/to/file");
        assert_eq!(uri.to_string(), "lakefs://repo/branch/path/to/file");
        
        let uri_no_path = LakeFSUri::new("repo", "branch");
        assert_eq!(uri_no_path.to_string(), "lakefs://repo/branch");
    }
}
