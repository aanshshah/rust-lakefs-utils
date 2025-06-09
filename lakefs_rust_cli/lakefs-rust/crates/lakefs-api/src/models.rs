use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repository {
    pub id: String,
    pub storage_namespace: String,
    pub default_branch: String,
    pub creation_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Branch {
    pub id: String,
    pub commit_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Commit {
    pub id: String,
    pub parents: Vec<String>,
    pub committer: String,
    pub message: String,
    pub creation_date: DateTime<Utc>,
    pub meta_range_id: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectStats {
    pub path: String,
    pub path_type: PathType,
    pub physical_address: String,
    pub checksum: String,
    pub size_bytes: i64,
    pub mtime: DateTime<Utc>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PathType {
    Object,
    Directory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffResult {
    pub results: Vec<Diff>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Diff {
    pub path: String,
    pub path_type: PathType,
    #[serde(rename = "type")]
    pub diff_type: DiffType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DiffType {
    Added,
    Removed,
    Changed,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffType::Added => write!(f, "added"),
            DiffType::Removed => write!(f, "removed"),
            DiffType::Changed => write!(f, "changed"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeResult {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination<T> {
    pub results: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub has_more: bool,
    pub max_per_page: i32,
    pub next_offset: Option<String>,
    pub results: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_repository_serialization() {
        let repo = Repository {
            id: "test-repo".to_string(),
            storage_namespace: "s3://bucket/path".to_string(),
            default_branch: "main".to_string(),
            creation_date: Utc::now(),
        };

        let json = serde_json::to_string(&repo).unwrap();
        let deserialized: Repository = serde_json::from_str(&json).unwrap();

        assert_eq!(repo.id, deserialized.id);
        assert_eq!(repo.storage_namespace, deserialized.storage_namespace);
        assert_eq!(repo.default_branch, deserialized.default_branch);
    }

    #[test]
    fn test_diff_type_display() {
        assert_eq!(DiffType::Added.to_string(), "added");
        assert_eq!(DiffType::Removed.to_string(), "removed");
        assert_eq!(DiffType::Changed.to_string(), "changed");
    }

    #[test]
    fn test_path_type_serialization() {
        let obj = PathType::Object;
        let dir = PathType::Directory;

        assert_eq!(serde_json::to_string(&obj).unwrap(), "\"object\"");
        assert_eq!(serde_json::to_string(&dir).unwrap(), "\"directory\"");
    }
}
