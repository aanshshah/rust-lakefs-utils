use crate::cli::FsCommands;
use crate::config::OptionsConfig;
use crate::utils::{parse_uri, format_size};
use anyhow::Result;
use bytes::Bytes;
use indicatif::{ProgressBar, ProgressStyle};
use lakefs_api::{LakeFSClient, models::PathType};
use std::path::Path;
use tabled::{Table, Tabled};
use tokio::fs;

#[derive(Tabled)]
struct ObjectRow {
    #[tabled(rename = "Type")]
    path_type: String,
    path: String,
    size: String,
    modified: String,
}

pub async fn execute(
    command: FsCommands,
    client: LakeFSClient,
    options: &OptionsConfig,
) -> Result<()> {
    match command {
        FsCommands::Ls { path, recursive: _ } => {
            let uri = parse_uri(&path)?;
            let response = client.list_objects(
                &uri.repository,
                &uri.reference,
                uri.path.as_deref(),
            ).await?;
            
            let rows: Vec<ObjectRow> = response
                .results
                .into_iter()
                .map(|obj| ObjectRow {
                    path_type: match obj.path_type {
                        PathType::Directory => "dir".to_string(),
                        PathType::Object => "file".to_string(),
                    },
                    path: obj.path,
                    size: format_size(obj.size_bytes),
                    modified: obj.mtime.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();
            
            let table = Table::new(rows);
            println!("{}", table);
        }
        
        FsCommands::Download {
            source,
            destination,
            recursive: _,
            parallelism: _,
        } => {
            let uri = parse_uri(&source)?;
            let path = uri.path.ok_or_else(|| {
                anyhow::anyhow!("Source URI must include a path")
            })?;
            
            let destination = destination.unwrap_or_else(|| {
                Path::new(&path).file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone())
            });
            
            // Download the object
            let pb = if !options.no_progress {
                let pb = ProgressBar::new(0);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} Downloading {msg}")
                        .unwrap(),
                );
                pb.set_message(path.clone());
                Some(pb)
            } else {
                None
            };
            
            let data = client.download_object(
                &uri.repository,
                &uri.reference,
                &path,
            ).await?;
            
            // Write to file
            fs::write(&destination, &data).await?;
            
            if let Some(pb) = pb {
                pb.finish_with_message(format!("Downloaded {} to {}", path, destination));
            } else {
                println!("Downloaded {} to {}", path, destination);
            }
        }
        
        FsCommands::Upload {
            source,
            destination,
            recursive,
            parallelism: _,
        } => {
            let uri = parse_uri(&destination)?;
            let path = uri.path.ok_or_else(|| {
                anyhow::anyhow!("Destination URI must include a path")
            })?;
            
            // Check if source exists
            let metadata = fs::metadata(&source).await?;
            
            if metadata.is_dir() && !recursive {
                anyhow::bail!("Source is a directory. Use -r/--recursive to upload directories");
            }
            
            // Read file content
            let pb = if !options.no_progress {
                let pb = ProgressBar::new(metadata.len());
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} Uploading {msg}")
                        .unwrap(),
                );
                pb.set_message(source.clone());
                Some(pb)
            } else {
                None
            };
            
            let data = fs::read(&source).await?;
            
            // Upload the object
            let stats = client.upload_object(
                &uri.repository,
                &uri.reference,
                &path,
                Bytes::from(data),
            ).await?;
            
            if let Some(pb) = pb {
                pb.finish_with_message(format!("Uploaded {} to {}", source, path));
            } else {
                println!("Uploaded {} to {}", source, path);
            }
            
            println!("Size: {}", format_size(stats.size_bytes));
            println!("Checksum: {}", stats.checksum);
        }
        
        FsCommands::Rm { path, recursive: _ } => {
            let uri = parse_uri(&path)?;
            let object_path = uri.path.ok_or_else(|| {
                anyhow::anyhow!("Path URI must include an object path")
            })?;
            
            client.delete_object(
                &uri.repository,
                &uri.reference,
                &object_path,
            ).await?;
            
            println!("Removed: {}", object_path);
        }
        
        FsCommands::Stat { path } => {
            let uri = parse_uri(&path)?;
            let object_path = uri.path.ok_or_else(|| {
                anyhow::anyhow!("Path URI must include an object path")
            })?;
            
            let stats = client.get_object(
                &uri.repository,
                &uri.reference,
                &object_path,
            ).await?;
            
            println!("Path: {}", stats.path);
            println!("Type: {:?}", stats.path_type);
            println!("Size: {}", format_size(stats.size_bytes));
            println!("Modified: {}", stats.mtime.format("%Y-%m-%d %H:%M:%S"));
            println!("Checksum: {}", stats.checksum);
            
            if let Some(metadata) = stats.metadata {
                println!("Metadata:");
                for (key, value) in metadata {
                    println!("  {}: {}", key, value);
                }
            }
        }
    }
    
    Ok(())
}