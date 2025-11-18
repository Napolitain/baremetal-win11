//! State persistence for crash recovery

use crate::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_STATE_AGE_SECS: u64 = 3600; // 1 hour

/// Frozen process information for persistence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrozenProcess {
    pub pid: u32,
    pub name: String,
    pub exe_path: String,
    pub timestamp: u64,
}

impl FrozenProcess {
    pub fn new(pid: u32, name: String, exe_path: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            pid,
            name,
            exe_path,
            timestamp,
        }
    }

    /// Check if this frozen process is stale (too old)
    pub fn is_stale(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.timestamp > MAX_STATE_AGE_SECS
    }
}

/// Persistent state container
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistentState {
    pub frozen_processes: Vec<FrozenProcess>,
}

impl PersistentState {
    pub fn new() -> Self {
        Self {
            frozen_processes: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frozen_processes.is_empty()
    }

    pub fn add(&mut self, pid: u32, name: String, exe_path: String) {
        self.frozen_processes
            .push(FrozenProcess::new(pid, name, exe_path));
    }

    pub fn remove(&mut self, pid: u32) {
        self.frozen_processes.retain(|p| p.pid != pid);
    }

    pub fn clear(&mut self) {
        self.frozen_processes.clear();
    }

    /// Get only non-stale processes
    pub fn get_valid_processes(&self) -> Vec<&FrozenProcess> {
        self.frozen_processes
            .iter()
            .filter(|p| !p.is_stale())
            .collect()
    }
}

impl Default for PersistentState {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for state persistence (allows different storage backends)
pub trait StatePersistence: Send + Sync {
    fn save(&self, state: &PersistentState) -> Result<()>;
    fn load(&self) -> Result<Option<PersistentState>>;
    fn delete(&self) -> Result<()>;
}

/// File-based state persistence
pub struct FileStatePersistence {
    path: PathBuf,
}

impl FileStatePersistence {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("smartfreeze_state.json");
        path
    }

    pub fn with_default_path() -> Self {
        Self::new(Self::default_path())
    }
}

impl StatePersistence for FileStatePersistence {
    fn save(&self, state: &PersistentState) -> Result<()> {
        let json = serde_json::to_string_pretty(state)?;
        fs::write(&self.path, json)?;
        Ok(())
    }

    fn load(&self) -> Result<Option<PersistentState>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.path)?;
        let state: PersistentState = serde_json::from_str(&content)?;
        Ok(Some(state))
    }

    fn delete(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frozen_process_creation() {
        let process = FrozenProcess::new(1234, "test.exe".to_string(), "C:\\test.exe".to_string());
        assert_eq!(process.pid, 1234);
        assert_eq!(process.name, "test.exe");
        assert_eq!(process.exe_path, "C:\\test.exe");
        assert!(process.timestamp > 0);
        assert!(!process.is_stale());
    }

    #[test]
    fn test_frozen_process_stale_detection() {
        // Create a process with old timestamp
        let mut process =
            FrozenProcess::new(1234, "test.exe".to_string(), "C:\\test.exe".to_string());
        process.timestamp = 0; // Very old timestamp

        assert!(process.is_stale());
    }

    #[test]
    fn test_persistent_state_add_remove() {
        let mut state = PersistentState::new();
        assert!(state.is_empty());

        state.add(1234, "test.exe".to_string(), "C:\\test.exe".to_string());
        assert!(!state.is_empty());
        assert_eq!(state.frozen_processes.len(), 1);

        state.add(
            5678,
            "another.exe".to_string(),
            "C:\\another.exe".to_string(),
        );
        assert_eq!(state.frozen_processes.len(), 2);

        state.remove(1234);
        assert_eq!(state.frozen_processes.len(), 1);
        assert_eq!(state.frozen_processes[0].pid, 5678);

        state.clear();
        assert!(state.is_empty());
    }

    #[test]
    fn test_persistent_state_valid_processes() {
        let mut state = PersistentState::new();

        // Add fresh process
        state.add(1234, "fresh.exe".to_string(), "C:\\fresh.exe".to_string());

        // Add stale process
        let mut stale =
            FrozenProcess::new(5678, "stale.exe".to_string(), "C:\\stale.exe".to_string());
        stale.timestamp = 0;
        state.frozen_processes.push(stale);

        let valid = state.get_valid_processes();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].pid, 1234);
    }

    #[test]
    fn test_file_persistence_save_load() {
        let temp_path = std::env::temp_dir().join("smartfreeze_test_state.json");
        let persistence = FileStatePersistence::new(temp_path.clone());

        // Clean up before test
        let _ = persistence.delete();

        // Create and save state
        let mut state = PersistentState::new();
        state.add(1234, "test.exe".to_string(), "C:\\test.exe".to_string());
        state.add(
            5678,
            "another.exe".to_string(),
            "C:\\another.exe".to_string(),
        );

        persistence.save(&state).unwrap();

        // Load and verify
        let loaded = persistence.load().unwrap();
        assert!(loaded.is_some());
        let loaded_state = loaded.unwrap();
        assert_eq!(loaded_state.frozen_processes.len(), 2);
        assert_eq!(loaded_state.frozen_processes[0].pid, 1234);
        assert_eq!(loaded_state.frozen_processes[1].pid, 5678);

        // Clean up
        persistence.delete().unwrap();
    }

    #[test]
    fn test_file_persistence_load_nonexistent() {
        let temp_path = std::env::temp_dir().join("smartfreeze_nonexistent.json");
        let persistence = FileStatePersistence::new(temp_path.clone());

        // Ensure file doesn't exist
        let _ = persistence.delete();

        let loaded = persistence.load().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_file_persistence_delete() {
        let temp_path = std::env::temp_dir().join("smartfreeze_test_delete.json");
        let persistence = FileStatePersistence::new(temp_path.clone());

        // Create file
        let state = PersistentState::new();
        persistence.save(&state).unwrap();
        assert!(temp_path.exists());

        // Delete
        persistence.delete().unwrap();
        assert!(!temp_path.exists());

        // Delete again (should not error)
        assert!(persistence.delete().is_ok());
    }
}
