use crate::error::{Error, Result};
use crate::index::{LocalIndex, IndexEntry};
use crate::changes::{Change, ChangeType, ChangeDetector};
use lakefs_api::{LakeFSClient, LakeFSUri, models::ObjectStats};
use bytes::Bytes;
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use tokio::fs;
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct SyncConfig {
    pub parallelism: usize,
    pub show_progress: bool,
    pub ignore_permissions: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            parallelism: 10,
            show_progress: true,
            ignore_permissions: true,
        }
    }
}

pub struct SyncManager {
    client: LakeFSClient,
    config: SyncConfig,
}

impl SyncManager {
    pub fn new(client: LakeFSClient, config: SyncConfig) -> Self {
        Self { client, config }
    }
    
    pub async fn sync(
        &self,
        local_path: &Path,
        remote: &LakeFSUri,
    ) -> Result<SyncResult> {
        // Load or create index
        let mut index = match LocalIndex::load(local_path) {
            Ok(idx) => idx,
            Err(_) => {
                // Get current commit
                let branch = self.client.get_branch(&remote.repository, &remote.reference).await?;
                LocalIndex::new(&remote.repository, &remote.reference, &branch.commit_id)
            }
        };
        
        // Get remote objects
        let remote_objects = self.list_remote_objects(remote).await?;
        
        // Detect changes
        let detector = ChangeDetector::new(local_path.to_path_buf());
        let changes = detector.detect_changes(&index, remote_objects)?;
        
        // Progress bar
        let pb = if self.config.show_progress {
            let pb = ProgressBar::new(changes.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            Some(pb)
        } else {
            None
        };
        
        // Process changes
        let semaphore = Arc::new(Semaphore::new(self.config.parallelism));
        let mut tasks = Vec::new();
        
        for change in changes {
            let client = self.client.clone();
            let remote = remote.clone();
            let local_path = local_path.to_path_buf();
            let sem = semaphore.clone();
            let pb = pb.clone();
            
            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let result = Self::process_change(&client, &change, &local_path, &remote).await;
                
                if let Some(pb) = pb {
                    pb.inc(1);
                }
                
                (change, result)
            });
            
            tasks.push(task);
        }
        
        // Collect results
        let mut uploaded = 0;
        let mut downloaded = 0;
        let mut removed = 0;
        let mut errors = Vec::new();
        
        for task in tasks {
            let (change, result) = task.await.map_err(|e| Error::Sync(e.to_string()))?;
            
            match result {
                Ok(entry) => {
                    match change.change_type {
                        ChangeType::Added | ChangeType::Modified => {
                            if change.local_path.is_some() {
                                uploaded += 1;
                            } else {
                                downloaded += 1;
                            }
                            index.add_entry(change.path, entry);
                        }
                        ChangeType::Removed => {
                            removed += 1;
                            index.remove_entry(&change.path);
                        }
                    }
                }
                Err(e) => {
                    errors.push((change.path, e));
                }
            }
        }
        
        if let Some(pb) = pb {
            pb.finish_with_message("Sync complete");
        }
        
        // Update index
        let branch = self.client.get_branch(&remote.repository, &remote.reference).await?;
        index.update_head(&branch.commit_id);
        index.save(local_path)?;
        
        Ok(SyncResult {
            uploaded,
            downloaded,
            removed,
            errors,
        })
    }
    
    async fn list_remote_objects(&self, remote: &LakeFSUri) -> Result<Vec<ObjectStats>> {
        let mut objects = Vec::new();
        
        let response = self.client.list_objects(
            &remote.repository,
            &remote.reference,
            remote.path.as_deref(),
        ).await?;
        
        objects.extend(response.results);
        
        // Handle pagination if needed
        // TODO: Implement pagination handling
        
        Ok(objects)
    }
    
    async fn process_change(
        client: &LakeFSClient,
        change: &Change,
        local_base: &Path,
        remote: &LakeFSUri,
    ) -> Result<IndexEntry> {
        match change.change_type {
            ChangeType::Added | ChangeType::Modified => {
                if let Some(local_path) = &change.local_path {
                    // Upload file
                    let data = fs::read(local_path).await?;
                    let _metadata = fs::metadata(local_path).await?;
                    
                    let remote_path = remote.path.as_ref().map_or(
                        change.path.clone(),
                        |p| format!("{}/{}", p, change.path),
                    );
                    
                    let stats = client.upload_object(
                        &remote.repository,
                        &remote.reference,
                        &remote_path,
                        Bytes::from(data),
                    ).await?;
                    
                    Ok(IndexEntry {
                        path: change.path.clone(),
                        checksum: stats.checksum,
                        size: stats.size_bytes as u64,
                        mtime: stats.mtime,
                        permissions: None,
                    })
                } else if let Some(remote_stats) = &change.remote_stats {
                    // Download file
                    let local_path = local_base.join(&change.path);
                    
                    // Create parent directory if needed
                    if let Some(parent) = local_path.parent() {
                        fs::create_dir_all(parent).await?;
                    }
                    
                    let data = client.download_object(
                        &remote.repository,
                        &remote.reference,
                        &remote_stats.path,
                    ).await?;
                    
                    fs::write(&local_path, &data).await?;
                    
                    Ok(IndexEntry {
                        path: change.path.clone(),
                        checksum: remote_stats.checksum.clone(),
                        size: remote_stats.size_bytes as u64,
                        mtime: remote_stats.mtime,
                        permissions: None,
                    })
                } else {
                    Err(Error::Sync("No source for change".into()))
                }
            }
            ChangeType::Removed => {
                if change.local_path.is_none() {
                    // Remove local file
                    let local_path = local_base.join(&change.path);
                    if local_path.exists() {
                        fs::remove_file(&local_path).await?;
                    }
                } else {
                    // Remove remote file
                    let remote_path = remote.path.as_ref().map_or(
                        change.path.clone(),
                        |p| format!("{}/{}", p, change.path),
                    );
                    
                    client.delete_object(
                        &remote.repository,
                        &remote.reference,
                        &remote_path,
                    ).await?;
                }
                
                Ok(IndexEntry {
                    path: change.path.clone(),
                    checksum: String::new(),
                    size: 0,
                    mtime: Utc::now(),
                    permissions: None,
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct SyncResult {
    pub uploaded: usize,
    pub downloaded: usize,
    pub removed: usize,
    pub errors: Vec<(String, Error)>,
}