use crate::error::{Error, Result};
use crate::index::{LocalIndex, IndexEntry};
use lakefs_api::models::ObjectStats;
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use chrono::{DateTime, Utc};
use ignore::gitignore::{Gitignore, GitignoreBuilder};

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub path: String,
    pub change_type: ChangeType,
    pub local_path: Option<PathBuf>,
    pub remote_stats: Option<ObjectStats>,
}

pub struct ChangeDetector {
    local_path: PathBuf,
    gitignore: Gitignore,
}

impl ChangeDetector {
    pub fn new(local_path: PathBuf) -> Self {
        let gitignore_path = local_path.join(".gitignore");
        let gitignore = if gitignore_path.exists() {
            let mut builder = GitignoreBuilder::new(&local_path);
            match builder.add(&gitignore_path) {
                None => builder.build().unwrap_or_else(|_| Gitignore::empty()),
                Some(_) => Gitignore::empty(), // Error adding gitignore
            }
        } else {
            Gitignore::empty()
        };
        
        Self {
            local_path,
            gitignore,
        }
    }
    
    pub fn detect_changes(
        &self,
        index: &LocalIndex,
        remote_objects: Vec<ObjectStats>,
    ) -> Result<Vec<Change>> {
        let mut changes = Vec::new();
        let mut remote_map: std::collections::HashMap<String, ObjectStats> = remote_objects
            .into_iter()
            .map(|obj| (obj.path.clone(), obj))
            .collect();
        
        // Check local files against index
        for entry in WalkDir::new(&self.local_path)
            .into_iter()
            .filter_entry(|e| !self.is_ignored(e.path()))
            .filter_map(|res| res.ok())  // Fixed: directly handle walkdir::Error
        {
            if !entry.file_type().is_file() {
                continue;
            }
            
            let relative_path = self.get_relative_path(entry.path())?;
            let metadata = fs::metadata(entry.path())?;
            
            match index.get_entry(&relative_path) {
                Some(index_entry) => {
                    // Check if file has been modified
                    if self.has_changed(entry.path(), index_entry, &metadata)? {
                        changes.push(Change {
                            path: relative_path.clone(),
                            change_type: ChangeType::Modified,
                            local_path: Some(entry.path().to_path_buf()),
                            remote_stats: remote_map.remove(&relative_path),
                        });
                    }
                }
                None => {
                    // New file
                    changes.push(Change {
                        path: relative_path.clone(),
                        change_type: ChangeType::Added,
                        local_path: Some(entry.path().to_path_buf()),
                        remote_stats: remote_map.remove(&relative_path),
                    });
                }
            }
        }
        
        // Check for removed files (in index but not on disk)
        for (path, _) in &index.entries {
            let local_path = self.local_path.join(path);
            if !local_path.exists() {
                changes.push(Change {
                    path: path.clone(),
                    change_type: ChangeType::Removed,
                    local_path: None,
                    remote_stats: remote_map.remove(path),
                });
            }
        }
        
        // Check for remote changes
        for (path, stats) in remote_map {
            match index.get_entry(&path) {
                Some(index_entry) => {
                    if stats.checksum != index_entry.checksum {
                        changes.push(Change {
                            path: path.clone(),
                            change_type: ChangeType::Modified,
                            local_path: None,
                            remote_stats: Some(stats),
                        });
                    }
                }
                None => {
                    changes.push(Change {
                        path: path.clone(),
                        change_type: ChangeType::Added,
                        local_path: None,
                        remote_stats: Some(stats),
                    });
                }
            }
        }
        
        Ok(changes)
    }
    
    fn is_ignored(&self, path: &Path) -> bool {
        if path.file_name().map(|n| n.to_str().unwrap_or("")).unwrap_or("").starts_with('.') {
            return true;
        }
        self.gitignore.matched(path, path.is_dir()).is_ignore()
    }
    
    fn get_relative_path(&self, path: &Path) -> Result<String> {
        path.strip_prefix(&self.local_path)
            .map_err(|_| Error::InvalidPath(format!("Path not within local directory: {:?}", path)))
            .map(|p| p.to_string_lossy().to_string())
    }
    
    fn has_changed(
        &self,
        path: &Path,
        index_entry: &IndexEntry,
        metadata: &fs::Metadata,
    ) -> Result<bool> {
        // Check size first (quick check)
        if metadata.len() != index_entry.size {
            return Ok(true);
        }
        
        // Check mtime (may not be reliable)
        let mtime: DateTime<Utc> = metadata.modified()?.into();
        if mtime > index_entry.mtime {
            // Verify with checksum
            let checksum = self.calculate_checksum(path)?;
            return Ok(checksum != index_entry.checksum);
        }
        
        Ok(false)
    }
    
    fn calculate_checksum(&self, path: &Path) -> Result<String> {
        let data = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use lakefs_api::models::PathType;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_change_type_equality()  {
        assert_eq!(ChangeType::Added, ChangeType::Added);
        assert_ne!(ChangeType::Added, ChangeType::Modified);
    }

    #[test]
    fn test_change_creation()  {
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
    fn test_change_detector_new()  {
        let temp_dir = TempDir::new().unwrap();
        let detector = ChangeDetector::new(temp_dir.path().to_path_buf());
        
        assert_eq!(detector.local_path, temp_dir.path());
    }

    #[test]
    fn test_is_ignored()  {
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
    fn test_get_relative_path()  {
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
    fn test_calculate_checksum()  {
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
    fn test_detect_changes_new_file()  {
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
    fn test_detect_changes_removed_file()  {
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
    fn test_detect_changes_modified_file()  {
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
    fn test_detect_remote_changes()  {
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
        
    }
}