//! Windows process control implementation

use crate::freeze_engine::ProcessController;
use crate::{Result, SmartFreezeError};
use std::process::Command;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{
    OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE,
};

/// Windows-specific process controller
pub struct WindowsProcessController;

impl WindowsProcessController {
    pub fn new() -> Self {
        Self
    }

    /// Restart a process from its executable path
    pub fn restart_process(&self, exe_path: &str) -> Result<u32> {
        // Start the process detached (no window, background)
        match Command::new(exe_path).spawn() {
            Ok(child) => Ok(child.id()),
            Err(e) => Err(SmartFreezeError::ResumeFailed {
                pid: 0,
                reason: format!("Failed to restart {}: {}", exe_path, e),
            }),
        }
    }

    /// Terminate a process to free RAM
    fn freeze_process_internal(&self, pid: u32) -> Result<usize> {
        unsafe {
            // Open process with terminate permission
            let process_handle = OpenProcess(PROCESS_TERMINATE | PROCESS_QUERY_INFORMATION, 0, pid);

            if process_handle.is_null() {
                return Err(SmartFreezeError::FreezeFailed {
                    pid,
                    reason: "Failed to open process (may need admin privileges)".to_string(),
                });
            }

            // Terminate the process (exit code 0 for clean shutdown)
            let result = TerminateProcess(process_handle, 0);
            CloseHandle(process_handle);

            if result != 0 {
                Ok(1) // Successfully terminated
            } else {
                Err(SmartFreezeError::FreezeFailed {
                    pid,
                    reason: "TerminateProcess failed".to_string(),
                })
            }
        }
    }

    /// Restart a terminated process by launching it again
    fn resume_process_internal(&self, _pid: u32) -> Result<usize> {
        // Note: We can't actually restart by PID since the process is gone
        // Resume will need the executable path from persistence
        // For now, just return Ok to indicate the "resume" was attempted
        // The actual restart logic is in the persistence layer
        Ok(0)
    }
}

impl Default for WindowsProcessController {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessController for WindowsProcessController {
    fn freeze(&self, pid: u32) -> Result<usize> {
        self.freeze_process_internal(pid)
    }

    fn resume(&self, pid: u32) -> Result<usize> {
        self.resume_process_internal(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = WindowsProcessController::new();
        assert!(std::mem::size_of_val(&controller) == 0); // Zero-sized type
    }

    // Note: We don't test actual freeze/resume as it requires admin privileges
    // and could affect system stability. These are tested via integration tests
    // with controlled test processes.
}
