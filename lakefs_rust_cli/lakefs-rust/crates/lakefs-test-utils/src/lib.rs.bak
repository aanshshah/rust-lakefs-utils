use lakefs_api::models::{Repository, Branch, Commit, ObjectStats, PathType};
use chrono::Utc;
use std::collections::HashMap;

pub mod fixtures {
    use super::*;
    
    pub fn test_repository() -> Repository {
        Repository {
            id: "test-repo".to_string(),
            storage_namespace: "s3://test-bucket".to_string(),
            default_branch: "main".to_string(),
            creation_date: Utc::now(),
        }
    }
    
    pub fn test_branch() -> Branch {
        Branch {
            id: "test-branch".to_string(),
            commit_id: "abc123".to_string(),
        }
    }
    
    pub fn test_commit() -> Commit {
        Commit {
            id: "abc123".to_string(),
            parents: vec!["parent1".to_string()],
            committer: "test@example.com".to_string(),
            message: "Test commit".to_string(),
            creation_date: Utc::now(),
            meta_range_id: "meta123".to_string(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn test_object_stats() -> ObjectStats {
        ObjectStats {
            path: "test/file.txt".to_string(),
            path_type: PathType::Object,
            physical_address: "s3://bucket/object".to_string(),
            checksum: "sha256:123abc".to_string(),
            size_bytes: 1024,
            mtime: Utc::now(),
            metadata: None,
        }
    }
}

pub mod mock_server {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use serde_json::json;
    
    pub async fn setup_mock_lakefs() -> MockServer {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/repositories"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "results": [],
                    "pagination": {
                        "has_more": false,
                        "max_per_page": 100,
                        "results": 0
                    }
                })))
            .mount(&mock_server)
            .await;
            
        mock_server
    }
}

pub mod test_helpers {
    use tempfile::TempDir;
    use std::fs;
    use std::path::Path;
    
    pub fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> std::io::Result<()> {
        let file_path = dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)
    }
    
    pub fn setup_test_directory() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        create_test_file(temp_dir.path(), "file1.txt", b"content1").unwrap();
        create_test_file(temp_dir.path(), "dir/file2.txt", b"content2").unwrap();
        create_test_file(temp_dir.path(), ".hidden", b"hidden").unwrap();
        
        temp_dir
    }
}
