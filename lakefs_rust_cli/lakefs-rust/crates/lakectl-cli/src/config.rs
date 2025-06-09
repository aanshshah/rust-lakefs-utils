use crate::cli::Cli;
use anyhow::{Context, Result};
use config::{Config, Environment, File};
use lakefs_auth::AuthConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub credentials: AuthConfig,
    #[serde(default)]
    pub options: OptionsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub endpoint_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsConfig {
    pub parallelism: usize,
    pub no_progress: bool,
}

impl Default for OptionsConfig {
    fn default() -> Self {
        Self {
            parallelism: 10,
            no_progress: false,
        }
    }
}

pub fn load_config(cli: &Cli) -> Result<AppConfig> {
    let mut builder = Config::builder();
    
    // Default config file location
    let default_config = dirs::config_dir()
        .map(|p| p.join("lakectl").join("config.yaml"))
        .or_else(|| dirs::home_dir().map(|p| p.join(".lakectl.yaml")));
    
    // Load from config file
    let config_file = cli.config
        .as_ref()
        .map(PathBuf::from)
        .or(default_config);
    
    if let Some(path) = config_file {
        if path.exists() {
            builder = builder.add_source(File::from(path));
        }
    }
    
    // Override with environment variables
    builder = builder.add_source(
        Environment::with_prefix("LAKECTL")
            .separator("_")
            .try_parsing(true),
    );
    
    // Build config
    let config = builder
        .build()
        .context("Failed to build configuration")?;
    
    // Parse into our structure
    config
        .try_deserialize()
        .context("Failed to deserialize configuration")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Commands, RepoCommands};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_options_config_default()  {
        let options = OptionsConfig::default();
        assert_eq!(options.parallelism, 10);
        assert!(!options.no_progress);
    }

    #[test]
    fn test_server_config()  {
        let server = ServerConfig {
            endpoint_url: "http://localhost:8000".to_string(),
        };
        assert_eq!(server.endpoint_url, "http://localhost:8000");
    }

    #[test]
    fn test_app_config_serialization()  {
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
    fn test_load_config_from_file()  {
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
    }
}