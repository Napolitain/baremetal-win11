//! Core freeze engine logic

use crate::categorization::ProcessCategorizer;
use crate::process::{ProcessCategory, ProcessInfo};
use crate::Result;

/// Configuration for the freeze engine
#[derive(Debug, Clone)]
pub struct FreezeConfig {
    /// Minimum memory threshold in MB
    pub min_memory_mb: u64,
    /// Whether to keep communication apps running
    pub keep_communication: bool,
}

impl Default for FreezeConfig {
    fn default() -> Self {
        Self {
            min_memory_mb: 100,
            keep_communication: false,
        }
    }
}

/// Trait for process enumeration (allows mocking)
pub trait ProcessEnumerator: Send + Sync {
    fn enumerate(&mut self) -> Result<Vec<ProcessInfo>>;
    fn get_foreground_pid(&self) -> Option<u32>;
}

/// Trait for process control (allows mocking)
pub trait ProcessController: Send + Sync {
    fn freeze(&self, pid: u32) -> Result<usize>;
    fn resume(&self, pid: u32) -> Result<usize>;
}

/// Main freeze engine coordinating process management
pub struct FreezeEngine<E, C, Cat>
where
    E: ProcessEnumerator,
    C: ProcessController,
    Cat: ProcessCategorizer,
{
    enumerator: E,
    controller: C,
    #[allow(dead_code)]
    categorizer: Cat,
    config: FreezeConfig,
}

impl<E, C, Cat> FreezeEngine<E, C, Cat>
where
    E: ProcessEnumerator,
    C: ProcessController,
    Cat: ProcessCategorizer,
{
    pub fn new(enumerator: E, controller: C, categorizer: Cat, config: FreezeConfig) -> Self {
        Self {
            enumerator,
            controller,
            categorizer,
            config,
        }
    }

    /// Get all running processes
    pub fn enumerate_processes(&mut self) -> Result<Vec<ProcessInfo>> {
        self.enumerator.enumerate()
    }

    /// Get foreground process ID
    pub fn get_foreground_pid(&self) -> Option<u32> {
        self.enumerator.get_foreground_pid()
    }

    /// Find processes that are safe to freeze
    pub fn find_safe_to_freeze(&mut self) -> Result<Vec<ProcessInfo>> {
        let processes = self.enumerator.enumerate()?;

        Ok(processes
            .into_iter()
            .filter(|p| {
                p.memory_mb >= self.config.min_memory_mb
                    && p.is_safe_to_freeze(self.config.keep_communication)
            })
            .collect())
    }

    /// Find all gaming processes
    pub fn find_gaming_processes(&mut self) -> Result<Vec<ProcessInfo>> {
        let processes = self.enumerator.enumerate()?;

        Ok(processes
            .into_iter()
            .filter(|p| p.category == ProcessCategory::Gaming)
            .collect())
    }

    /// Freeze a specific process
    pub fn freeze_process(&self, pid: u32) -> Result<usize> {
        self.controller.freeze(pid)
    }

    /// Resume a specific process
    pub fn resume_process(&self, pid: u32) -> Result<usize> {
        self.controller.resume(pid)
    }

    /// Freeze multiple processes, returning PIDs of successfully frozen processes
    pub fn freeze_multiple(&self, pids: &[u32]) -> Vec<(u32, Result<usize>)> {
        pids.iter()
            .map(|&pid| (pid, self.freeze_process(pid)))
            .collect()
    }

    /// Resume multiple processes
    pub fn resume_multiple(&self, pids: &[u32]) -> Vec<(u32, Result<usize>)> {
        pids.iter()
            .map(|&pid| (pid, self.resume_process(pid)))
            .collect()
    }

    /// Get current configuration
    pub fn config(&self) -> &FreezeConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: FreezeConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::categorization::DefaultCategorizer;

    struct MockEnumerator {
        processes: Vec<ProcessInfo>,
        foreground_pid: Option<u32>,
    }

    impl MockEnumerator {
        fn new(processes: Vec<ProcessInfo>, foreground_pid: Option<u32>) -> Self {
            Self {
                processes,
                foreground_pid,
            }
        }
    }

    impl ProcessEnumerator for MockEnumerator {
        fn enumerate(&mut self) -> Result<Vec<ProcessInfo>> {
            Ok(self.processes.clone())
        }

        fn get_foreground_pid(&self) -> Option<u32> {
            self.foreground_pid
        }
    }

    struct MockController {
        frozen_pids: std::sync::Arc<std::sync::Mutex<Vec<u32>>>,
    }

    impl MockController {
        fn new() -> Self {
            Self {
                frozen_pids: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn get_frozen_pids(&self) -> Vec<u32> {
            self.frozen_pids.lock().unwrap().clone()
        }
    }

    impl ProcessController for MockController {
        fn freeze(&self, pid: u32) -> Result<usize> {
            self.frozen_pids.lock().unwrap().push(pid);
            Ok(1)
        }

        fn resume(&self, pid: u32) -> Result<usize> {
            let mut pids = self.frozen_pids.lock().unwrap();
            pids.retain(|&p| p != pid);
            Ok(1)
        }
    }

    fn create_test_process(
        pid: u32,
        name: &str,
        memory_mb: u64,
        is_foreground: bool,
        category: ProcessCategory,
    ) -> ProcessInfo {
        ProcessInfo::new(
            pid,
            name.to_string(),
            format!("C:\\Test\\{}", name),
            memory_mb,
            is_foreground,
            category,
        )
    }

    #[test]
    fn test_find_safe_to_freeze_filters_by_memory() {
        let processes = vec![
            create_test_process(1, "low_mem.exe", 50, false, ProcessCategory::Productivity),
            create_test_process(2, "high_mem.exe", 200, false, ProcessCategory::Productivity),
        ];

        let enumerator = MockEnumerator::new(processes, None);
        let controller = MockController::new();
        let categorizer = DefaultCategorizer::new();
        let config = FreezeConfig {
            min_memory_mb: 100,
            keep_communication: false,
        };

        let mut engine = FreezeEngine::new(enumerator, controller, categorizer, config);
        let safe = engine.find_safe_to_freeze().unwrap();

        assert_eq!(safe.len(), 1);
        assert_eq!(safe[0].pid, 2);
    }

    #[test]
    fn test_find_safe_to_freeze_excludes_foreground() {
        let processes = vec![
            create_test_process(
                1,
                "foreground.exe",
                200,
                true,
                ProcessCategory::Productivity,
            ),
            create_test_process(
                2,
                "background.exe",
                200,
                false,
                ProcessCategory::Productivity,
            ),
        ];

        let enumerator = MockEnumerator::new(processes, Some(1));
        let controller = MockController::new();
        let categorizer = DefaultCategorizer::new();
        let config = FreezeConfig::default();

        let mut engine = FreezeEngine::new(enumerator, controller, categorizer, config);
        let safe = engine.find_safe_to_freeze().unwrap();

        assert_eq!(safe.len(), 1);
        assert_eq!(safe[0].pid, 2);
    }

    #[test]
    fn test_find_safe_to_freeze_excludes_critical() {
        let processes = vec![
            create_test_process(1, "explorer.exe", 200, false, ProcessCategory::Critical),
            create_test_process(2, "chrome.exe", 200, false, ProcessCategory::Productivity),
        ];

        let enumerator = MockEnumerator::new(processes, None);
        let controller = MockController::new();
        let categorizer = DefaultCategorizer::new();
        let config = FreezeConfig::default();

        let mut engine = FreezeEngine::new(enumerator, controller, categorizer, config);
        let safe = engine.find_safe_to_freeze().unwrap();

        assert_eq!(safe.len(), 1);
        assert_eq!(safe[0].pid, 2);
    }

    #[test]
    fn test_keep_communication_flag() {
        let processes = vec![
            create_test_process(1, "discord.exe", 200, false, ProcessCategory::Communication),
            create_test_process(2, "chrome.exe", 200, false, ProcessCategory::Productivity),
        ];

        let enumerator = MockEnumerator::new(processes.clone(), None);
        let controller = MockController::new();
        let categorizer = DefaultCategorizer::new();
        let config = FreezeConfig {
            min_memory_mb: 100,
            keep_communication: false,
        };

        let mut engine = FreezeEngine::new(enumerator, controller, categorizer, config);
        let safe = engine.find_safe_to_freeze().unwrap();
        assert_eq!(safe.len(), 2); // Both freezable

        // Now with keep_communication enabled
        let enumerator2 = MockEnumerator::new(processes, None);
        let controller2 = MockController::new();
        let categorizer2 = DefaultCategorizer::new();
        let config2 = FreezeConfig {
            min_memory_mb: 100,
            keep_communication: true,
        };

        let mut engine2 = FreezeEngine::new(enumerator2, controller2, categorizer2, config2);
        let safe2 = engine2.find_safe_to_freeze().unwrap();
        assert_eq!(safe2.len(), 1); // Only chrome, discord protected
        assert_eq!(safe2[0].pid, 2);
    }

    #[test]
    fn test_freeze_multiple() {
        let processes = vec![];
        let enumerator = MockEnumerator::new(processes, None);
        let controller = MockController::new();
        let categorizer = DefaultCategorizer::new();
        let config = FreezeConfig::default();

        let engine = FreezeEngine::new(enumerator, controller, categorizer, config);

        let results = engine.freeze_multiple(&[1, 2, 3]);
        assert_eq!(results.len(), 3);

        for (_pid, result) in results {
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 1);
        }

        let frozen = engine.controller.get_frozen_pids();
        assert_eq!(frozen, vec![1, 2, 3]);
    }

    #[test]
    fn test_find_gaming_processes() {
        let processes = vec![
            create_test_process(1, "steam.exe", 150, false, ProcessCategory::Gaming),
            create_test_process(2, "game.exe", 500, false, ProcessCategory::Gaming),
            create_test_process(3, "chrome.exe", 200, false, ProcessCategory::Productivity),
        ];

        let enumerator = MockEnumerator::new(processes, None);
        let controller = MockController::new();
        let categorizer = DefaultCategorizer::new();
        let config = FreezeConfig::default();

        let mut engine = FreezeEngine::new(enumerator, controller, categorizer, config);
        let gaming = engine.find_gaming_processes().unwrap();

        assert_eq!(gaming.len(), 2);
        assert!(gaming.iter().any(|p| p.pid == 1));
        assert!(gaming.iter().any(|p| p.pid == 2));
    }
}
