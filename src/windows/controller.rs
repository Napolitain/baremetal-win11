//! Windows process control implementation

use crate::freeze_engine::ProcessController;
use crate::{Result, SmartFreezeError};
use std::mem;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::Threading::{
    OpenThread, ResumeThread, SuspendThread, PROCESS_SUSPEND_RESUME,
};

/// Windows-specific process controller
pub struct WindowsProcessController;

impl WindowsProcessController {
    pub fn new() -> Self {
        Self
    }

    /// Freeze a process by suspending all its threads
    fn freeze_process_internal(&self, pid: u32) -> Result<usize> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot.is_null() || snapshot == (-1isize) as HANDLE {
                return Err(SmartFreezeError::FreezeFailed {
                    pid,
                    reason: "Failed to create thread snapshot".to_string(),
                });
            }

            let mut entry: THREADENTRY32 = mem::zeroed();
            entry.dwSize = mem::size_of::<THREADENTRY32>() as u32;
            let mut suspended_count = 0;

            if Thread32First(snapshot, &mut entry) != 0 {
                loop {
                    if entry.th32OwnerProcessID == pid {
                        let thread_handle = OpenThread(PROCESS_SUSPEND_RESUME, 0, entry.th32ThreadID);
                        if !thread_handle.is_null() {
                            if SuspendThread(thread_handle) != u32::MAX {
                                suspended_count += 1;
                            }
                            CloseHandle(thread_handle);
                        }
                    }
                    if Thread32Next(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);

            if suspended_count > 0 {
                Ok(suspended_count)
            } else {
                Err(SmartFreezeError::FreezeFailed {
                    pid,
                    reason: format!("Failed to suspend any threads for PID {}", pid),
                })
            }
        }
    }

    /// Resume a frozen process by resuming all its threads
    fn resume_process_internal(&self, pid: u32) -> Result<usize> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot.is_null() || snapshot == (-1isize) as HANDLE {
                return Err(SmartFreezeError::ResumeFailed {
                    pid,
                    reason: "Failed to create thread snapshot".to_string(),
                });
            }

            let mut entry: THREADENTRY32 = mem::zeroed();
            entry.dwSize = mem::size_of::<THREADENTRY32>() as u32;
            let mut resumed_count = 0;

            if Thread32First(snapshot, &mut entry) != 0 {
                loop {
                    if entry.th32OwnerProcessID == pid {
                        let thread_handle = OpenThread(PROCESS_SUSPEND_RESUME, 0, entry.th32ThreadID);
                        if !thread_handle.is_null() {
                            if ResumeThread(thread_handle) != u32::MAX {
                                resumed_count += 1;
                            }
                            CloseHandle(thread_handle);
                        }
                    }
                    if Thread32Next(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);

            if resumed_count > 0 {
                Ok(resumed_count)
            } else {
                Err(SmartFreezeError::ResumeFailed {
                    pid,
                    reason: format!("Failed to resume any threads for PID {}", pid),
                })
            }
        }
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
