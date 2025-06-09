use crate::{auth_provider::AuthProvider, error::{Error, Result}};
use async_trait::async_trait;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_credential_types::provider::{SharedCredentialsProvider, ProvideCredentials};
use aws_sigv4::http_request::{sign, SignableBody, SignableRequest, SigningSettings};
use aws_sigv4::sign::v4;
use aws_types::region::Region;
use chrono::Utc;
use http::{Method, Uri};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
struct AwsAuthRequest {
    method: String,
    host: String,
    region: String,
    service: String,
    date: String,
    expires_in: i64,
    body: String,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LakeFSAuthResponse {
    token: String,
}

pub struct AwsIamAuth {
    region: Region,
    endpoint: String,
    base_uri: Option<String>,
    credentials_provider: SharedCredentialsProvider,
    client: Client,
}

impl AwsIamAuth {
    pub async fn new(
        region: String,
        endpoint: &str,
        base_uri: Option<String>,
    ) -> Result<Self> {
        let region_provider = RegionProviderChain::default_provider()
            .or_else(Region::new(region.clone()));
        
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        
        Ok(Self {
            region: Region::new(region),
            endpoint: endpoint.to_string(),
            base_uri,
            credentials_provider: config.credentials_provider()
                .ok_or_else(|| Error::Config("No AWS credentials provider found".into()))?,
            client: Client::new(),
        })
    }
    
    async fn create_sts_request(&self) -> Result<AwsAuthRequest> {
        let credentials = self.credentials_provider
            .provide_credentials()
            .await
            .map_err(|e| Error::Aws(e.to_string()))?;
        
        // Create STS GetCallerIdentity request
        let method = Method::POST;
        let service = "sts";
        let host = format!("sts.{}.amazonaws.com", self.region.as_ref());
        let uri = Uri::builder()
            .scheme("https")
            .authority(host.as_str())
            .path_and_query("/?Action=GetCallerIdentity&Version=2011-06-15")
            .build()
            .map_err(|e| Error::Config(e.to_string()))?;
        
        let now = SystemTime::now();
        let signing_settings = SigningSettings::default();
        
        let body = "";
        let signable_body = SignableBody::Bytes(body.as_bytes());
        
        let mut headers = http::HeaderMap::new();
        headers.insert("Host", host.parse().unwrap());
        
        // Convert headers to the format expected by SignableRequest
        let header_vec: Vec<(&str, &str)> = headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.to_str().unwrap()))
            .collect();
        
        let mut request = http::Request::builder()
            .method(method.clone())
            .uri(&uri)
            .body(body.as_bytes().to_vec())
            .map_err(|e| Error::Config(e.to_string()))?;
        
        *request.headers_mut() = headers.clone();
        
        // Create a signable request
        let signable_request = SignableRequest::new(
            method.as_str(),
            uri.to_string(),
            header_vec.into_iter(),
            signable_body,
        ).map_err(|e| Error::Aws(format!("Failed to create signable request: {}", e)))?;
        
        // Create signing parameters
        let identity = aws_credential_types::Credentials::new(
            credentials.access_key_id(),
            credentials.secret_access_key(),
            credentials.session_token().map(|s| s.to_string()),
            None,
            "manual",
        ).into();
        
        let signing_params = v4::SigningParams::builder()
            .identity(&identity)
            .region(self.region.as_ref())
            .name(service)
            .time(now)
            .settings(signing_settings)
            .build()
            .map_err(|e| Error::Aws(format!("Failed to build signing params: {}", e)))?
            .into();
        
        // Sign the request
        let output = sign(signable_request, &signing_params)
            .map_err(|e| Error::Aws(e.to_string()))?;
        
        // Extract signed headers directly from the output
        let mut header_map = std::collections::HashMap::new();
        
        // The output contains the signature - let's extract it properly
        let (_signing_instructions, signature) = output.into_parts();
        
        // Add the Authorization header with the signature
        header_map.insert("Authorization".to_string(), signature);
        
        // Add the X-Amz-Date header
        let date_time = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        header_map.insert("X-Amz-Date".to_string(), date_time);
        
        // Add security token if present
        if let Some(token) = credentials.session_token() {
            header_map.insert("X-Amz-Security-Token".to_string(), token.to_string());
        }
        
        // Add other required headers
        for (name, value) in headers.iter() {
            if !header_map.contains_key(name.as_str()) {
                header_map.insert(
                    name.to_string(),
                    value.to_str().map_err(|_| Error::Config("Invalid header value".into()))?.to_string(),
                );
            }
        }
        
        Ok(AwsAuthRequest {
            method: method.to_string(),
            host: host.clone(),
            region: self.region.to_string(),
            service: service.to_string(),
            date: Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
            expires_in: 900,
            body: body.to_string(),
            headers: header_map,
        })
    }
    
    async fn get_lakefs_token(&self, auth_request: AwsAuthRequest) -> Result<String> {
        let url = match &self.base_uri {
            Some(base) => format!("{}/external/auth/external_principal_login", base),
            None => format!("{}/api/v1/external/auth/external_principal_login", self.endpoint),
        };
        
        let response = self.client
            .post(&url)
            .json(&auth_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let auth_response: LakeFSAuthResponse = response.json().await?;
            Ok(auth_response.token)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Aws(format!("Authentication failed: {}", error_text)))
        }
    }
}

#[async_trait]
impl AuthProvider for AwsIamAuth {
    async fn get_auth_header(&self) -> Result<String> {
        let sts_request = self.create_sts_request().await?;
        let token = self.get_lakefs_token(sts_request).await?;
        Ok(format!("Bearer {}", token))
    }
}