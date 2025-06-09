use crate::{error::{Error, Result}, models::*};
use bytes::Bytes;
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct LakeFSClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl LakeFSClient {
    pub fn new(base_url: impl Into<String>, auth_header: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            auth_header: auth_header.into(),
        }
    }
    
    pub fn with_client(client: Client, base_url: impl Into<String>, auth_header: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(), 
            auth_header: auth_header.into(),
        }
    }
    
    async fn request<T: DeserializeOwned>(&self, method: reqwest::Method, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client
            .request(method, &url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
            
        self.handle_response(response).await
    }
    
    async fn request_with_body<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client
            .request(method, &url)
            .header("Authorization", &self.auth_header)
            .json(body)
            .send()
            .await?;
            
        self.handle_response(response).await
    }
    
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            match status {
                StatusCode::NOT_FOUND => Err(Error::NotFound(message)),
                StatusCode::UNAUTHORIZED => Err(Error::Auth(message)),
                _ => Err(Error::Api {
                    status: status.as_u16(),
                    message,
                }),
            }
        }
    }
    
    // Repository operations
    pub async fn create_repository(&self, name: &str, storage_namespace: &str) -> Result<Repository> {
        let body = serde_json::json!({
            "name": name,
            "storage_namespace": storage_namespace,
        });
        
        self.request_with_body(reqwest::Method::POST, "/repositories", &body).await
    }
    
    pub async fn list_repositories(&self) -> Result<Pagination<Repository>> {
        self.request(reqwest::Method::GET, "/repositories").await
    }
    
    pub async fn get_repository(&self, repository: &str) -> Result<Repository> {
        let path = format!("/repositories/{}", repository);
        self.request(reqwest::Method::GET, &path).await
    }
    
    pub async fn delete_repository(&self, repository: &str) -> Result<()> {
        let path = format!("/repositories/{}", repository);
        let _: serde_json::Value = self.request(reqwest::Method::DELETE, &path).await?;
        Ok(())
    }
    
    // Branch operations  
    pub async fn create_branch(&self, repository: &str, branch: &str, source: &str) -> Result<Branch> {
        let path = format!("/repositories/{}/branches", repository);
        let body = serde_json::json!({
            "name": branch,
            "source": source,
        });
        
        self.request_with_body(reqwest::Method::POST, &path, &body).await
    }
    
    pub async fn list_branches(&self, repository: &str) -> Result<Pagination<Branch>> {
        let path = format!("/repositories/{}/branches", repository);
        self.request(reqwest::Method::GET, &path).await
    }
    
    pub async fn get_branch(&self, repository: &str, branch: &str) -> Result<Branch> {
        let path = format!("/repositories/{}/branches/{}", repository, branch);
        self.request(reqwest::Method::GET, &path).await
    }
    
    pub async fn delete_branch(&self, repository: &str, branch: &str) -> Result<()> {
        let path = format!("/repositories/{}/branches/{}", repository, branch);
        let _: serde_json::Value = self.request(reqwest::Method::DELETE, &path).await?;
        Ok(())
    }
    
    // Commit operations
    pub async fn commit(&self, repository: &str, branch: &str, message: &str) -> Result<Commit> {
        let path = format!("/repositories/{}/branches/{}/commits", repository, branch);
        let body = serde_json::json!({
            "message": message,
        });
        
        self.request_with_body(reqwest::Method::POST, &path, &body).await
    }
    
    pub async fn get_commit(&self, repository: &str, commit_id: &str) -> Result<Commit> {
        let path = format!("/repositories/{}/commits/{}", repository, commit_id);
        self.request(reqwest::Method::GET, &path).await
    }
    
    pub async fn log_commits(&self, repository: &str, branch: &str) -> Result<Pagination<Commit>> {
        let path = format!("/repositories/{}/branches/{}/commits", repository, branch);
        self.request(reqwest::Method::GET, &path).await
    }
    
    // Object operations
    pub async fn list_objects(&self, repository: &str, reference: &str, path: Option<&str>) -> Result<Pagination<ObjectStats>> {
        let base_path = format!("/repositories/{}/refs/{}/objects", repository, reference);
        let path = match path {
            Some(p) => format!("{}?prefix={}", base_path, p),
            None => base_path,
        };
        
        self.request(reqwest::Method::GET, &path).await
    }
    
    pub async fn get_object(&self, repository: &str, reference: &str, path: &str) -> Result<ObjectStats> {
        let path = format!("/repositories/{}/refs/{}/objects/stat?path={}", repository, reference, path);
        self.request(reqwest::Method::GET, &path).await
    }
    
    pub async fn upload_object(
        &self,
        repository: &str,
        branch: &str,
        path: &str,
        content: Bytes,
    ) -> Result<ObjectStats> {
        let url = format!("{}/repositories/{}/branches/{}/objects?path={}", 
                         self.base_url, repository, branch, path);
        
        let response = self.client
            .put(&url)
            .header("Authorization", &self.auth_header)
            .body(content)
            .send()
            .await?;
            
        self.handle_response(response).await
    }
    
    pub async fn download_object(&self, repository: &str, reference: &str, path: &str) -> Result<Bytes> {
        let url = format!("{}/repositories/{}/refs/{}/objects?path={}", 
                         self.base_url, repository, reference, path);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        let status = response.status();  // Capture status before consuming response
        
        if status.is_success() {
            Ok(response.bytes().await?)
        } else {
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Api {
                status: status.as_u16(),
                message,
            })
        }
    }
    
    pub async fn delete_object(&self, repository: &str, branch: &str, path: &str) -> Result<()> {
        let url = format!("{}/repositories/{}/branches/{}/objects?path={}", 
                         self.base_url, repository, branch, path);
        
        let response = self.client
            .delete(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        let status = response.status();  // Capture status before consuming response
            
        if status.is_success() {
            Ok(())
        } else {
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Api {
                status: status.as_u16(),
                message,
            })
        }
    }
    
    // Diff operations
    pub async fn diff(&self, repository: &str, left_ref: &str, right_ref: &str) -> Result<DiffResult> {
        let path = format!("/repositories/{}/refs/{}/diff/{}", repository, left_ref, right_ref);
        self.request(reqwest::Method::GET, &path).await
    }
    
    // Merge operations
    pub async fn merge(&self, repository: &str, source_ref: &str, destination_branch: &str) -> Result<MergeResult> {
        let path = format!("/repositories/{}/refs/{}/merge/{}", repository, source_ref, destination_branch);
        let body = serde_json::json!({});
        
        self.request_with_body(reqwest::Method::POST, &path, &body).await
    }
}

    use wiremock::matchers::{method, path, header};
    use wiremock::MockServer;

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