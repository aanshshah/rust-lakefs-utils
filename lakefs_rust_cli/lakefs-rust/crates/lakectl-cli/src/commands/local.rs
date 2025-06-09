use crate::cli::LocalCommands;
use crate::config::OptionsConfig;
use anyhow::Result;
use lakefs_api::{LakeFSClient, LakeFSUri};
use lakefs_local::{SyncManager, SyncConfig, LocalIndex};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub async fn execute(
    command: LocalCommands,
    client: LakeFSClient,
    options: &OptionsConfig,
) -> Result<()> {
    match command {
        LocalCommands::Init { remote, path } => {
            let uri = LakeFSUri::from_str(&remote)?;
            let path = Path::new(&path);
            
            // Check if already initialized
            if LocalIndex::load(path).is_ok() {
                anyhow::bail!("Directory already initialized");
            }
            
            // Get current commit
            let branch = client.get_branch(&uri.repository, &uri.reference).await?;
            
            // Create index
            let index = LocalIndex::new(
                &uri.repository,
                &uri.reference,
                &branch.commit_id,
            );
            
            index.save(path)?;
            println!("Initialized local directory at {}", path.display());
        }
        
        LocalCommands::Clone { remote, path } => {
            let uri = LakeFSUri::from_str(&remote)?;
            let path = path.map(PathBuf::from).unwrap_or_else(|| {
                PathBuf::from(&uri.repository)
            });
            
            // Check if directory exists
            if path.exists() {
                anyhow::bail!("Directory already exists: {}", path.display());
            }
            
            // Create directory
            std::fs::create_dir_all(&path)?;
            
            // Initialize index
            let branch = client.get_branch(&uri.repository, &uri.reference).await?;
            let index = LocalIndex::new(
                &uri.repository,
                &uri.reference,
                &branch.commit_id,
            );
            
            index.save(&path)?;
            
            // Perform initial sync
            let config = SyncConfig {
                parallelism: options.parallelism,
                show_progress: !options.no_progress,
                ..Default::default()
            };
            
            let sync_manager = SyncManager::new(client, config);
            let result = sync_manager.sync(&path, &uri).await?;
            
            println!(
                "Cloned {} to {}",
                uri.to_string(),
                path.display()
            );
            println!(
                "Downloaded: {}, Errors: {}",
                result.downloaded,
                result.errors.len()
            );
        }
        
        LocalCommands::Status { path } => {
            let path = Path::new(&path);
            let index = LocalIndex::load(path)?;
            
            println!("Repository: {}", index.repository);
            println!("Branch: {}", index.reference);
            println!("Head commit: {}", index.head_commit);
            println!("Last sync: {}", index.last_sync.format("%Y-%m-%d %H:%M:%S"));
            println!("Tracked files: {}", index.entries.len());
        }
        
        LocalCommands::Pull { path, force: _ } => {
            let path = Path::new(&path);
            let mut index = LocalIndex::load(path)?;
            
            let uri = LakeFSUri::new(&index.repository, &index.reference);
            
            let config = SyncConfig {
                parallelism: options.parallelism,
                show_progress: !options.no_progress,
                ..Default::default()
            };
            
            let sync_manager = SyncManager::new(client.clone(), config);
            let result = sync_manager.sync(path, &uri).await?;
            
            // Update index with new head
            let branch = client.get_branch(&uri.repository, &uri.reference).await?;
            index.update_head(&branch.commit_id);
            index.save(path)?;
            
            println!(
                "Pulled changes from {}",
                uri.to_string()
            );
            println!(
                "Downloaded: {}, Uploaded: {}, Removed: {}, Errors: {}",
                result.downloaded,
                result.uploaded,
                result.removed,
                result.errors.len()
            );
        }
        
        LocalCommands::Commit { path, message } => {
            if message.trim().is_empty() {
                anyhow::bail!("Commit message cannot be empty");
            }
            
            let path = Path::new(&path);
            let mut index = LocalIndex::load(path)?;
            
            let uri = LakeFSUri::new(&index.repository, &index.reference);
            
            // First sync to upload changes
            let config = SyncConfig {
                parallelism: options.parallelism,
                show_progress: !options.no_progress,
                ..Default::default()
            };
            
            let sync_manager = SyncManager::new(client.clone(), config);
            let result = sync_manager.sync(path, &uri).await?;
            
            if result.uploaded == 0 && result.removed == 0 {
                println!("No changes to commit");
                return Ok(());
            }
            
            // Create commit
            let commit = client.commit(
                &uri.repository,
                &uri.reference,
                &message,
            ).await?;
            
            // Update index with new head
            index.update_head(&commit.id);
            index.save(path)?;
            
            println!("Created commit: {}", commit.id);
            println!("Message: {}", commit.message);
            println!(
                "Uploaded: {}, Removed: {}",
                result.uploaded,
                result.removed
            );
        }
    }
    
    Ok(())
}