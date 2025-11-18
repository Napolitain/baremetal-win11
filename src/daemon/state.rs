//! Daemon state management

use std::collections::HashSet;

/// Daemon runtime state
#[derive(Debug)]
pub struct DaemonState {
    /// PIDs of currently frozen processes
    pub frozen_pids: HashSet<u32>,
    /// Whether a game is currently running
    pub game_detected: bool,
    /// Whether auto-freeze is enabled
    pub enabled: bool,
}

impl DaemonState {
    pub fn new() -> Self {
        Self {
            frozen_pids: HashSet::new(),
            game_detected: false,
            enabled: true,
        }
    }

    pub fn add_frozen(&mut self, pid: u32) {
        self.frozen_pids.insert(pid);
    }

    pub fn clear_frozen(&mut self) -> Vec<u32> {
        self.frozen_pids.drain().collect()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_state_creation() {
        let state = DaemonState::new();
        assert!(state.enabled);
        assert!(!state.game_detected);
        assert!(state.frozen_pids.is_empty());
    }

    #[test]
    fn test_add_frozen() {
        let mut state = DaemonState::new();
        state.add_frozen(1234);
        assert_eq!(state.frozen_pids.len(), 1);
        assert!(state.frozen_pids.contains(&1234));
    }

    #[test]
    fn test_clear_frozen() {
        let mut state = DaemonState::new();
        state.add_frozen(1234);
        state.add_frozen(5678);

        let pids = state.clear_frozen();
        assert_eq!(pids.len(), 2);
        assert!(state.frozen_pids.is_empty());
    }

    #[test]
    fn test_toggle_enabled() {
        let mut state = DaemonState::new();
        assert!(state.is_enabled());

        state.toggle_enabled();
        assert!(!state.is_enabled());

        state.toggle_enabled();
        assert!(state.is_enabled());
    }
}
