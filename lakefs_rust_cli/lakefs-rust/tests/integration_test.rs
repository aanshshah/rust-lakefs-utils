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
