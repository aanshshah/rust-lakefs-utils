#!/bin/bash

set -e

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

cd lakefs-rust

# Add comprehensive tests for lakefs-api/client.rs
print_info "Adding tests to lakefs-api/client.rs..."
cat >> crates/lakefs-api/src/client.rs << 'EOF'

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

    #[tokio::test]
    async fn test_client_creation() {
        let client = LakeFSClient::new("http://localhost:8000", "Bearer test-token");
        assert_eq!(client.base_url, "http://localhost:8000");
        assert_eq!(client.auth_header, "Bearer test-token");
    }

    #[tokio::test]
    async fn test_create_repository() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/repositories"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "id": "test-repo",
                    "storage_namespace": "s3://bucket",
                    "default_branch": "main",
                    "creation_date": "2024-01-01T00:00:00Z"
                })))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        let repo = client.create_repository("test-repo", "s3://bucket").await.unwrap();

        assert_eq!(repo.id, "test-repo");
        assert_eq!(repo.storage_namespace, "s3://bucket");
        assert_eq!(repo.default_branch, "main");
    }

    #[tokio::test]
    async fn test_list_repositories() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/repositories"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "results": [{
                        "id": "repo1",
                        "storage_namespace": "s3://bucket1",
                        "default_branch": "main",
                        "creation_date": "2024-01-01T00:00:00Z"
                    }],
                    "pagination": {
                        "has_more": false,
                        "max_per_page": 100,
                        "results": 1,
                        "next_offset": null
                    }
                })))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        let repos = client.list_repositories().await.unwrap();

        assert_eq!(repos.results.len(), 1);
        assert_eq!(repos.results[0].id, "repo1");
        assert!(!repos.pagination.has_more);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/repositories/nonexistent"))
            .respond_with(ResponseTemplate::new(404)
                .set_body_string("Repository not found"))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        let result = client.get_repository("nonexistent").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::NotFound(msg) => assert_eq!(msg, "Repository not found"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_auth_error() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/repositories"))
            .respond_with(ResponseTemplate::new(401)
                .set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer invalid-token");
        let result = client.list_repositories().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Auth(msg) => assert_eq!(msg, "Unauthorized"),
            _ => panic!("Expected Auth error"),
        }
    }

    #[tokio::test]
    async fn test_create_branch() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/repositories/test-repo/branches"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "id": "feature-branch",
                    "commit_id": "abc123"
                })))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        let branch = client.create_branch("test-repo", "feature-branch", "main").await.unwrap();

        assert_eq!(branch.id, "feature-branch");
        assert_eq!(branch.commit_id, "abc123");
    }

    #[tokio::test]
    async fn test_upload_download_object() {
        let mock_server = MockServer::start().await;
        
        // Mock upload
        Mock::given(method("PUT"))
            .and(path("/repositories/test-repo/branches/main/objects"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "path": "test.txt",
                    "path_type": "object",
                    "physical_address": "s3://bucket/object",
                    "checksum": "checksum123",
                    "size_bytes": 100,
                    "mtime": "2024-01-01T00:00:00Z"
                })))
            .mount(&mock_server)
            .await;

        // Mock download
        Mock::given(method("GET"))
            .and(path("/repositories/test-repo/refs/main/objects"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_bytes(b"test content"))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        
        // Test upload
        let stats = client.upload_object(
            "test-repo",
            "main",
            "test.txt",
            Bytes::from("test content"),
        ).await.unwrap();
        
        assert_eq!(stats.path, "test.txt");
        assert_eq!(stats.size_bytes, 100);
        
        // Test download
        let data = client.download_object("test-repo", "main", "test.txt").await.unwrap();
        assert_eq!(data, Bytes::from("test content"));
    }
}
EOF

# Add tests for lakefs-auth auth_provider.rs
print_info "Adding tests to lakefs-auth/auth_provider.rs..."
cat >> crates/lakefs-auth/src/auth_provider.rs << 'EOF'

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_basic_auth_provider() {
        let config = AuthConfig::Basic {
            access_key_id: "test-key".to_string(),
            secret_access_key: "test-secret".to_string(),
        };
        
        let provider = create_auth_provider(config, "http://localhost").await.unwrap();
        let header = provider.get_auth_header().await.unwrap();
        
        assert!(header.starts_with("Basic "));
    }

    #[tokio::test]
    async fn test_auth_config_serialization() {
        let basic_config = AuthConfig::Basic {
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
        };
        
        let json = serde_json::to_string(&basic_config).unwrap();
        assert!(json.contains("\"type\":\"Basic\""));
        
        let aws_config = AuthConfig::AwsIam {
            region: "us-east-1".to_string(),
            base_uri: Some("http://custom".to_string()),
        };
        
        let json = serde_json::to_string(&aws_config).unwrap();
        assert!(json.contains("\"type\":\"AwsIam\""));
    }
}
EOF

# Add tests for lakefs-local changes.rs
print_info "Adding tests to lakefs-local/changes.rs..."
cat >> crates/lakefs-local/src/changes.rs << 'EOF'

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_change_type_equality() {
        assert_eq!(ChangeType::Added, ChangeType::Added);
        assert_ne!(ChangeType::Added, ChangeType::Modified);
    }

    #[test]
    fn test_change_creation() {
        let change = Change {
            path: "test.txt".to_string(),
            change_type: ChangeType::Added,
            local_path: Some(PathBuf::from("/tmp/test.txt")),
            remote_stats: None,
        };
        
        assert_eq!(change.path, "test.txt");
        assert_eq!(change.change_type, ChangeType::Added);
        assert!(change.local_path.is_some());
    }

    #[test]
    fn test_change_detector_new() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        assert_eq!(detector.local_path, temp_dir.path());
    }

    #[test]
    fn test_is_ignored() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        // Hidden files should be ignored
        assert!(detector.is_ignored(Path::new(".hidden")));
        assert!(detector.is_ignored(Path::new("/path/to/.git")));
        
        // Normal files should not be ignored
        assert!(!detector.is_ignored(Path::new("normal.txt")));
        assert!(!detector.is_ignored(Path::new("/path/to/file.rs")));
    }

    #[test]
    fn test_get_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        let full_path = temp_dir.path().join("subdir/file.txt");
        let relative = detector.get_relative_path(&full_path).unwrap();
        
        assert_eq!(relative, "subdir/file.txt");
        
        // Path outside base should error
        let outside_path = Path::new("/some/other/path");
        assert!(detector.get_relative_path(outside_path).is_err());
    }

    #[test]
    fn test_calculate_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        // Create a test file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"Hello, world!").unwrap();
        
        let checksum = detector.calculate_checksum(&file_path).unwrap();
        
        // SHA256 of "Hello, world!"
        assert_eq!(checksum, "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3");
    }

    #[test]
    fn test_detect_changes_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        // Create a new file
        let file_path = temp_dir.path().join("new.txt");
        fs::write(&file_path, b"content").unwrap();
        
        // Empty index
        let index = LocalIndex::new("test", "main", "commit1");
        
        let changes = detector.detect_changes(&index, vec![]).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "new.txt");
        assert_eq!(changes[0].change_type, ChangeType::Added);
        assert!(changes[0].local_path.is_some());
    }

    #[test]
    fn test_detect_changes_removed_file() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        // Index with file that doesn't exist on disk
        let mut index = LocalIndex::new("test", "main", "commit1");
        index.add_entry("removed.txt".to_string(), IndexEntry {
            path: "removed.txt".to_string(),
            checksum: "abc123".to_string(),
            size: 100,
            mtime: Utc::now(),
            permissions: None,
        });
        
        let changes = detector.detect_changes(&index, vec![]).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "removed.txt");
        assert_eq!(changes[0].change_type, ChangeType::Removed);
        assert!(changes[0].local_path.is_none());
    }

    #[test]
    fn test_detect_changes_modified_file() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        // Create a file
        let file_path = temp_dir.path().join("modified.txt");
        fs::write(&file_path, b"original content").unwrap();
        
        // Index with different size
        let mut index = LocalIndex::new("test", "main", "commit1");
        index.add_entry("modified.txt".to_string(), IndexEntry {
            path: "modified.txt".to_string(),
            checksum: "different".to_string(),
            size: 50, // Different size than actual
            mtime: Utc::now() - chrono::Duration::days(1),
            permissions: None,
        });
        
        let changes = detector.detect_changes(&index, vec![]).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "modified.txt");
        assert_eq!(changes[0].change_type, ChangeType::Modified);
    }

    #[test]
    fn test_detect_remote_changes() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        let index = LocalIndex::new("test", "main", "commit1");
        
        // Remote object that doesn't exist locally
        let remote_objects = vec![ObjectStats {
            path: "remote-only.txt".to_string(),
            path_type: PathType::Object,
            physical_address: "s3://bucket/object".to_string(),
            checksum: "remote123".to_string(),
            size_bytes: 200,
            mtime: Utc::now(),
            metadata: None,
        }];
        
        let changes = detector.detect_changes(&index, remote_objects).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "remote-only.txt");
        assert_eq!(changes[0].change_type, ChangeType::Added);
        assert!(changes[0].remote_stats.is_some());
    }
}
EOF

# Add tests for lakectl-cli config.rs
print_info "Adding tests to lakectl-cli/config.rs..."
cat >> crates/lakectl-cli/src/config.rs << 'EOF'

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_options_config_default() {
        let options = OptionsConfig::default();
        assert_eq!(options.parallelism, 10);
        assert!(!options.no_progress);
    }

    #[test]
    fn test_server_config() {
        let server = ServerConfig {
            endpoint_url: "http://localhost:8000".to_string(),
        };
        assert_eq!(server.endpoint_url, "http://localhost:8000");
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig {
            server: ServerConfig {
                endpoint_url: "http://test.lakefs.io".to_string(),
            },
            credentials: AuthConfig::Basic {
                access_key_id: "test-key".to_string(),
                secret_access_key: "test-secret".to_string(),
            },
            options: OptionsConfig::default(),
        };
        
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("endpoint_url"));
        assert!(yaml.contains("test-key"));
        
        let deserialized: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.server.endpoint_url, "http://test.lakefs.io");
    }

    #[test]
    fn test_load_config_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        
        let config_content = r#"
server:
  endpoint_url: http://test.lakefs.io
credentials:
  type: Basic
  access_key_id: test_key
  secret_access_key: test_secret
options:
  parallelism: 20
  no_progress: true
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let cli = Cli {
            command: Commands::Repo { 
                command: RepoCommands::List { 
                    amount: None, 
                    after: None 
                } 
            },
            config: Some(config_path.to_string_lossy().to_string()),
            verbose: false,
            no_color: false,
        };
        
        let config = load_config(&cli).unwrap();
        
        assert_eq!(config.server.endpoint_url, "http://test.lakefs.io");
        assert_eq!(config.options.parallelism, 20);
        assert!(config.options.no_progress);
    }
}
EOF

# Add tests for CLI commands
print_info "Adding tests to lakectl-cli/commands/branch.rs..."
cat >> crates/lakectl-cli/src/commands/branch.rs << 'EOF'

#[cfg(test)]
mod tests {
    use super::*;
    use lakefs_test_utils::mock_server::setup_mock_lakefs;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_create_branch_command() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/repositories/test-repo/branches"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "id": "feature-branch",
                    "commit_id": "abc123"
                })))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        
        let command = BranchCommands::Create {
            uri: "lakefs://test-repo/feature-branch".to_string(),
            source: "main".to_string(),
        };
        
        let result = execute(command, client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_branches_command() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/repositories/test-repo/branches"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "results": [{
                        "id": "main",
                        "commit_id": "abc123"
                    }],
                    "pagination": {
                        "has_more": false,
                        "max_per_page": 100,
                        "results": 1,
                        "next_offset": null
                    }
                })))
            .mount(&mock_server)
            .await;

        let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
        
        let command = BranchCommands::List {
            repository: "lakefs://test-repo".to_string(),
            amount: None,
        };
        
        let result = execute(command, client).await;
        assert!(result.is_ok());
    }
}
EOF

# Add integration tests
print_info "Creating integration tests..."
cat > tests/integration_test.rs << 'EOF'
use lakefs_test_utils::mock_server::setup_mock_lakefs;
use lakefs_api::{LakeFSClient, LakeFSUri};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use std::str::FromStr;

#[tokio::test]
async fn test_client_integration() {
    let mock_server = setup_mock_lakefs().await;
    let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
    
    // Test listing repositories
    let repos = client.list_repositories().await.unwrap();
    assert_eq!(repos.results.len(), 0);
}

#[tokio::test]
async fn test_full_workflow() {
    let mock_server = MockServer::start().await;
    
    // Mock repository creation
    Mock::given(method("POST"))
        .and(path("/repositories"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "id": "test-repo",
                "storage_namespace": "s3://bucket",
                "default_branch": "main",
                "creation_date": "2024-01-01T00:00:00Z"
            })))
        .mount(&mock_server)
        .await;
    
    // Mock branch creation
    Mock::given(method("POST"))
        .and(path("/repositories/test-repo/branches"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "id": "feature",
                "commit_id": "abc123"
            })))
        .mount(&mock_server)
        .await;
    
    // Mock object upload
    Mock::given(method("PUT"))
        .and(path("/repositories/test-repo/branches/main/objects"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "path": "test.txt",
                "path_type": "object",
                "physical_address": "s3://bucket/object",
                "checksum": "checksum123",
                "size_bytes": 100,
                "mtime": "2024-01-01T00:00:00Z"
            })))
        .mount(&mock_server)
        .await;
    
    let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
    
    // Test workflow
    let repo = client.create_repository("test-repo", "s3://bucket").await.unwrap();
    assert_eq!(repo.id, "test-repo");
    
    let branch = client.create_branch("test-repo", "feature", "main").await.unwrap();
    assert_eq!(branch.id, "feature");
    
    let stats = client.upload_object(
        "test-repo",
        "main",
        "test.txt",
        bytes::Bytes::from("test content"),
    ).await.unwrap();
    assert_eq!(stats.path, "test.txt");
}

#[tokio::test]
async fn test_uri_parsing() {
    let uri = LakeFSUri::from_str("lakefs://repo/branch/path/to/file").unwrap();
    assert_eq!(uri.repository, "repo");
    assert_eq!(uri.reference, "branch");
    assert_eq!(uri.path, Some("path/to/file".to_string()));
}

#[tokio::test]
async fn test_error_propagation() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/repositories/nonexistent"))
        .respond_with(ResponseTemplate::new(404)
            .set_body_string("Not found"))
        .mount(&mock_server)
        .await;
    
    let client = LakeFSClient::new(mock_server.uri(), "Bearer test-token");
    let result = client.get_repository("nonexistent").await;
    
    assert!(result.is_err());
}
EOF

# Create a test coverage script
print_info "Creating test coverage script..."
cat > run_coverage.sh << 'EOF'
#!/bin/bash

set -e

echo "Installing tarpaulin if not installed..."
if ! command -v cargo-tarpaulin &> /dev/null; then
    cargo install cargo-tarpaulin
fi

echo "Running tests with coverage..."
cargo tarpaulin \
    --all-features \
    --workspace \
    --out Html \
    --out Lcov \
    --exclude-files '*/tests/*' \
    --exclude-files '*/target/*' \
    --ignore-panics \
    --ignore-tests

echo "Coverage report generated:"
echo "  - HTML: target/tarpaulin/tarpaulin-report.html"
echo "  - LCOV: target/lcov.info"

# Print summary
echo ""
echo "Coverage Summary:"
cargo tarpaulin --print-summary
EOF

chmod +x run_coverage.sh

print_status "Comprehensive tests added!"
echo ""
echo "Test coverage has been significantly improved. Run tests with:"
echo "  cargo test --all"
echo ""
echo "To generate a coverage report:"
echo "  ./run_coverage.sh"
echo ""
echo "New tests added:"
echo "  - lakefs-api: Client tests with mocked HTTP responses"
echo "  - lakefs-auth: Auth provider and config tests"
echo "  - lakefs-local: Change detection and sync tests"
echo "  - lakectl-cli: Config and command tests"
echo "  - Integration tests: Full workflow testing"
