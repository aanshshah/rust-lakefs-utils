use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexEntry {
    pub path: String,
    pub checksum: String,
    pub size: u64,
    pub mtime: DateTime<Utc>,
    pub permissions: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalIndex {
    pub version: u32,
    pub repository: String,
    pub reference: String,
    pub head_commit: String,
    pub entries: HashMap<String, IndexEntry>,
    pub last_sync: DateTime<Utc>,
}

impl LocalIndex {
    const INDEX_FILE: &'static str = ".lakectl/index.json";
    const VERSION: u32 = 1;
    
    pub fn new(repository: &str, reference: &str, head_commit: &str) -> Self {
        Self {
            version: Self::VERSION,
            repository: repository.to_string(),
            reference: reference.to_string(),
            head_commit: head_commit.to_string(),
            entries: HashMap::new(),
            last_sync: Utc::now(),
        }
    }
    
    pub fn load(path: &Path) -> Result<Self> {
        let index_path = path.join(Self::INDEX_FILE);
        let data = fs::read_to_string(&index_path)
            .map_err(|e| Error::Index(format!("Failed to read index: {}", e)))?;
        
        let index: Self = serde_json::from_str(&data)
            .map_err(|e| Error::Index(format!("Failed to parse index: {}", e)))?;
            
        if index.version != Self::VERSION {
            return Err(Error::Index(format!(
                "Unsupported index version: {}", 
                index.version
            )));
        }
        
        Ok(index)
    }
    
    pub fn save(&self, path: &Path) -> Result<()> {
        let index_path = path.join(Self::INDEX_FILE);
        
        // Create directory if it doesn't exist
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let data = serde_json::to_string_pretty(&self)
            .map_err(|e| Error::Index(format!("Failed to serialize index: {}", e)))?;
            
        fs::write(&index_path, data)?;
        Ok(())
    }
    
    pub fn get_entry(&self, path: &str) -> Option<&IndexEntry> {
        self.entries.get(path)
    }
    
    pub fn add_entry(&mut self, path: String, entry: IndexEntry) {
        self.entries.insert(path, entry);
    }
    
    pub fn remove_entry(&mut self, path: &str) -> Option<IndexEntry> {
        self.entries.remove(path)
    }
    
    pub fn update_head(&mut self, commit_id: &str) {
        self.head_commit = commit_id.to_string();
        self.last_sync = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_new_index() {
        let index = LocalIndex::new("test-repo", "main", "commit123");
        
        assert_eq!(index.version, LocalIndex::VERSION);
        assert_eq!(index.repository, "test-repo");
        assert_eq!(index.reference, "main");
        assert_eq!(index.head_commit, "commit123");
        assert!(index.entries.is_empty());
    }

    #[test]
    fn test_index_entry_operations() {
        let mut index = LocalIndex::new("test-repo", "main", "commit123");
        
        let entry = IndexEntry {
            path: "test.txt".to_string(),
            checksum: "abc123".to_string(),
            size: 1024,
            mtime: Utc::now(),
            permissions: Some(0o644),
        };
        
        // Add entry
        index.add_entry("test.txt".to_string(), entry.clone());
        assert_eq!(index.entries.len(), 1);
        
        // Get entry
        let retrieved = index.get_entry("test.txt").unwrap();
        assert_eq!(retrieved.checksum, "abc123");
        assert_eq!(retrieved.size, 1024);
        
        // Remove entry
        let removed = index.remove_entry("test.txt").unwrap();
        assert_eq!(removed.checksum, "abc123");
        assert!(index.entries.is_empty());
    }

    #[test]
    fn test_index_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        // Create and save index
        let mut index = LocalIndex::new("test-repo", "main", "commit123");
        index.add_entry("file1.txt".to_string(), IndexEntry {
            path: "file1.txt".to_string(),
            checksum: "checksum1".to_string(),
            size: 100,
            mtime: Utc::now(),
            permissions: None,
        });
        
        index.save(path).unwrap();
        
        // Load index
        let loaded = LocalIndex::load(path).unwrap();
        
        assert_eq!(loaded.repository, "test-repo");
        assert_eq!(loaded.reference, "main");
        assert_eq!(loaded.head_commit, "commit123");
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.get_entry("file1.txt").unwrap().checksum, "checksum1");
    }
}
