//! Process information and categorization types

use serde::{Deserialize, Serialize};

/// Process importance category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessCategory {
    /// Critical system processes (never freeze)
    Critical,
    /// Gaming-related processes (important to keep responsive)
    Gaming,
    /// Communication apps (potentially important)
    Communication,
    /// Background services and launchers (safe to freeze)
    BackgroundService,
    /// Browsers and productivity apps (safe to freeze when not foreground)
    Productivity,
    /// Unknown/uncategorized processes
    Unknown,
}

impl ProcessCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessCategory::Critical => "Critical",
            ProcessCategory::Gaming => "Gaming",
            ProcessCategory::Communication => "Communication",
            ProcessCategory::BackgroundService => "Background",
            ProcessCategory::Productivity => "Productivity",
            ProcessCategory::Unknown => "Unknown",
        }
    }
}

/// Represents a process with its resource usage
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub full_path: String,
    pub memory_mb: u64,
    pub cpu_percent: f64,
    pub is_foreground: bool,
    pub category: ProcessCategory,
}

impl ProcessInfo {
    pub fn new(
        pid: u32,
        name: String,
        full_path: String,
        memory_mb: u64,
        is_foreground: bool,
        category: ProcessCategory,
    ) -> Self {
        Self {
            pid,
            name,
            full_path,
            memory_mb,
            cpu_percent: 0.0,
            is_foreground,
            category,
        }
    }

    /// Check if this process is safe to freeze
    pub fn is_safe_to_freeze(&self, keep_communication: bool) -> bool {
        !self.is_foreground
            && self.category != ProcessCategory::Critical
            && self.category != ProcessCategory::Gaming
            && !(keep_communication && self.category == ProcessCategory::Communication)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_category_as_str() {
        assert_eq!(ProcessCategory::Critical.as_str(), "Critical");
        assert_eq!(ProcessCategory::Gaming.as_str(), "Gaming");
        assert_eq!(ProcessCategory::Communication.as_str(), "Communication");
    }

    #[test]
    fn test_process_is_safe_to_freeze_critical() {
        let process = ProcessInfo::new(
            1234,
            "explorer.exe".to_string(),
            "C:\\Windows\\explorer.exe".to_string(),
            200,
            false,
            ProcessCategory::Critical,
        );
        
        assert!(!process.is_safe_to_freeze(false));
        assert!(!process.is_safe_to_freeze(true));
    }

    #[test]
    fn test_process_is_safe_to_freeze_foreground() {
        let process = ProcessInfo::new(
            1234,
            "chrome.exe".to_string(),
            "C:\\Program Files\\Google\\Chrome\\chrome.exe".to_string(),
            500,
            true, // foreground
            ProcessCategory::Productivity,
        );
        
        assert!(!process.is_safe_to_freeze(false));
    }

    #[test]
    fn test_process_is_safe_to_freeze_communication() {
        let process = ProcessInfo::new(
            1234,
            "discord.exe".to_string(),
            "C:\\Discord\\discord.exe".to_string(),
            300,
            false,
            ProcessCategory::Communication,
        );
        
        assert!(process.is_safe_to_freeze(false)); // Can freeze if flag not set
        assert!(!process.is_safe_to_freeze(true)); // Protected when flag set
    }

    #[test]
    fn test_process_is_safe_to_freeze_background() {
        let process = ProcessInfo::new(
            1234,
            "googledrive.exe".to_string(),
            "C:\\Program Files\\Google\\Drive\\googledrive.exe".to_string(),
            150,
            false,
            ProcessCategory::BackgroundService,
        );
        
        assert!(process.is_safe_to_freeze(false));
        assert!(process.is_safe_to_freeze(true));
    }
}
