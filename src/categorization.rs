//! Process categorization logic

use crate::process::ProcessCategory;
use std::collections::HashMap;

/// Trait for categorizing processes
pub trait ProcessCategorizer: Send + Sync {
    /// Categorize a process based on its attributes
    fn categorize(&self, pid: u32, name: &str, path: &str) -> ProcessCategory;

    /// Check if a process is critical
    fn is_critical(&self, name: &str) -> bool;
}

/// Default implementation of process categorization
pub struct DefaultCategorizer {
    parent_map: HashMap<u32, u32>,
}

impl DefaultCategorizer {
    pub fn new() -> Self {
        Self {
            parent_map: HashMap::new(),
        }
    }

    pub fn update_parent_map(&mut self, pid: u32, parent_pid: u32) {
        if pid != 0 {
            self.parent_map.insert(pid, parent_pid);
        }
    }

    fn is_gaming_by_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();

        // Game launchers
        if name_lower.contains("steam")
            || name_lower.contains("epic")
            || name_lower.contains("origin")
            || name_lower.contains("gog")
            || name_lower.contains("battle.net")
            || name_lower.contains("battlenet")
            || name_lower.contains("uplay")
            || name_lower.contains("ubisoft")
        {
            return true;
        }

        // Anti-cheat
        if name_lower.contains("easyanticheat")
            || name_lower.contains("battleye")
            || name_lower.contains("vanguard")
        {
            return true;
        }

        // Common game patterns
        if name_lower.contains("game") && name_lower.contains(".exe") {
            return true;
        }

        false
    }

    fn is_gaming_by_path(&self, path: &str) -> bool {
        let path_lower = path.to_lowercase();

        let gaming_paths = [
            "\\steam\\",
            "\\steamapps\\",
            "\\steamlibrary\\",
            "\\epic games\\",
            "\\epicgames\\",
            "\\origin games\\",
            "\\gog galaxy\\",
            "\\gog games\\",
            "\\battle.net\\",
            "\\ubisoft\\",
            "\\ea games\\",
            "\\riot games\\",
            "\\games\\",
            "\\my games\\",
        ];

        gaming_paths
            .iter()
            .any(|&pattern| path_lower.contains(pattern))
    }

    fn is_communication(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();

        let communication_apps = [
            "discord",
            "slack",
            "teams",
            "telegram",
            "signal",
            "whatsapp",
            "zoom",
            "skype",
            "mumble",
            "teamspeak",
            "ventrilo",
            "element",
            "riot",
        ];

        communication_apps
            .iter()
            .any(|&app| name_lower.contains(app))
    }

    fn is_background_service(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();

        let background_services = [
            "updater",
            "update",
            "helper",
            "sync",
            "backup",
            "nvidia",
            "amd",
            "geforce",
            "radeon",
            "onedrive",
            "dropbox",
            "google drive",
            "toolbox",
        ];

        background_services
            .iter()
            .any(|&service| name_lower.contains(service))
    }

    fn is_productivity(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();

        let productivity_apps = [
            "chrome",
            "firefox",
            "edge",
            "opera",
            "brave",
            "vivaldi",
            "excel",
            "word",
            "powerpoint",
            "outlook",
            "onenote",
            "vscode",
            "code",
            "pycharm",
            "intellij",
            "rider",
            "sublime",
            "spotify",
            "vlc",
            "itunes",
            "notion",
            "obsidian",
        ];

        productivity_apps
            .iter()
            .any(|&app| name_lower.contains(app))
    }
}

impl Default for DefaultCategorizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessCategorizer for DefaultCategorizer {
    fn categorize(&self, _pid: u32, name: &str, path: &str) -> ProcessCategory {
        // Critical check first
        if self.is_critical(name) {
            return ProcessCategory::Critical;
        }

        // Gaming checks
        if self.is_gaming_by_path(path) || self.is_gaming_by_name(name) {
            return ProcessCategory::Gaming;
        }

        // Communication apps
        if self.is_communication(name) {
            return ProcessCategory::Communication;
        }

        // Background services
        if self.is_background_service(name) {
            return ProcessCategory::BackgroundService;
        }

        // Productivity apps
        if self.is_productivity(name) {
            return ProcessCategory::Productivity;
        }

        ProcessCategory::Unknown
    }

    fn is_critical(&self, name: &str) -> bool {
        let critical = [
            "system",
            "smss.exe",
            "csrss.exe",
            "wininit.exe",
            "services.exe",
            "lsass.exe",
            "svchost.exe",
            "winlogon.exe",
            "explorer.exe",
            "dwm.exe",
            "textinputhost.exe",
            "searchhost.exe",
            "startmenuexperiencehost.exe",
        ];

        critical.iter().any(|&c| name.eq_ignore_ascii_case(c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_process_detection() {
        let categorizer = DefaultCategorizer::new();

        assert!(categorizer.is_critical("explorer.exe"));
        assert!(categorizer.is_critical("Explorer.EXE"));
        assert!(categorizer.is_critical("textinputhost.exe"));
        assert!(!categorizer.is_critical("chrome.exe"));
    }

    #[test]
    fn test_gaming_detection_by_name() {
        let categorizer = DefaultCategorizer::new();

        assert_eq!(
            categorizer.categorize(1234, "steam.exe", "C:\\Program Files\\Steam\\steam.exe"),
            ProcessCategory::Gaming
        );

        assert_eq!(
            categorizer.categorize(1234, "game.exe", "C:\\Games\\game.exe"),
            ProcessCategory::Gaming
        );
    }

    #[test]
    fn test_gaming_detection_by_path() {
        let categorizer = DefaultCategorizer::new();

        assert_eq!(
            categorizer.categorize(
                1234,
                "MyGame.exe",
                "D:\\SteamLibrary\\steamapps\\common\\MyGame\\MyGame.exe"
            ),
            ProcessCategory::Gaming
        );

        assert_eq!(
            categorizer.categorize(
                1234,
                "Fortnite.exe",
                "C:\\Epic Games\\Fortnite\\Fortnite.exe"
            ),
            ProcessCategory::Gaming
        );
    }

    #[test]
    fn test_communication_detection() {
        let categorizer = DefaultCategorizer::new();

        assert_eq!(
            categorizer.categorize(1234, "Discord.exe", "C:\\Users\\User\\Discord\\Discord.exe"),
            ProcessCategory::Communication
        );

        assert_eq!(
            categorizer.categorize(1234, "Teams.exe", "C:\\Microsoft\\Teams\\Teams.exe"),
            ProcessCategory::Communication
        );
    }

    #[test]
    fn test_background_service_detection() {
        let categorizer = DefaultCategorizer::new();

        assert_eq!(
            categorizer.categorize(
                1234,
                "GoogleDriveSync.exe",
                "C:\\Program Files\\Google\\Drive\\GoogleDriveSync.exe"
            ),
            ProcessCategory::BackgroundService
        );

        assert_eq!(
            categorizer.categorize(
                1234,
                "jetbrains-toolbox.exe",
                "C:\\Users\\User\\AppData\\Local\\JetBrains\\Toolbox\\jetbrains-toolbox.exe"
            ),
            ProcessCategory::BackgroundService
        );
    }

    #[test]
    fn test_productivity_detection() {
        let categorizer = DefaultCategorizer::new();

        assert_eq!(
            categorizer.categorize(
                1234,
                "chrome.exe",
                "C:\\Program Files\\Google\\Chrome\\chrome.exe"
            ),
            ProcessCategory::Productivity
        );

        assert_eq!(
            categorizer.categorize(1234, "Code.exe", "C:\\Program Files\\VSCode\\Code.exe"),
            ProcessCategory::Productivity
        );
    }

    #[test]
    fn test_unknown_process() {
        let categorizer = DefaultCategorizer::new();

        assert_eq!(
            categorizer.categorize(1234, "unknown.exe", "C:\\Some\\Path\\unknown.exe"),
            ProcessCategory::Unknown
        );
    }
}
