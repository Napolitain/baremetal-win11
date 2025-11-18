//! Windows process enumeration implementation

use crate::categorization::{DefaultCategorizer, ProcessCategorizer};
use crate::freeze_engine::ProcessEnumerator;
use crate::process::{ProcessCategory, ProcessInfo};
use crate::{Result, SmartFreezeError};
use std::collections::HashMap;
use std::mem;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Windows-specific process enumerator
pub struct WindowsProcessEnumerator {
    categorizer: DefaultCategorizer,
    parent_map: HashMap<u32, u32>,
}

impl WindowsProcessEnumerator {
    pub fn new() -> Self {
        Self {
            categorizer: DefaultCategorizer::new(),
            parent_map: HashMap::new(),
        }
    }

    /// Get process name and path
    fn get_process_info(&self, pid: u32) -> (String, String) {
        unsafe {
            let process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                0,
                pid,
            );

            if process_handle.is_null() {
                return (String::new(), String::new());
            }

            let mut path_buffer: [u16; 260] = [0; 260];
            let mut path_len = path_buffer.len() as u32;

            if QueryFullProcessImageNameW(process_handle, 0, path_buffer.as_mut_ptr(), &mut path_len) != 0 {
                let full_path = String::from_utf16_lossy(&path_buffer[..path_len as usize]);
                let name = full_path
                    .rsplit('\\')
                    .next()
                    .unwrap_or("unknown.exe")
                    .to_string();

                CloseHandle(process_handle);
                (name, full_path)
            } else {
                CloseHandle(process_handle);
                (String::new(), String::new())
            }
        }
    }

    /// Get process memory usage in MB
    fn get_memory_usage(&self, pid: u32) -> u64 {
        unsafe {
            let process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                0,
                pid,
            );

            if process_handle.is_null() {
                return 0;
            }

            let mut pmc: PROCESS_MEMORY_COUNTERS = mem::zeroed();
            pmc.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

            if GetProcessMemoryInfo(process_handle, &mut pmc, pmc.cb) != 0 {
                CloseHandle(process_handle);
                pmc.WorkingSetSize as u64 / (1024 * 1024)
            } else {
                CloseHandle(process_handle);
                0
            }
        }
    }

    /// Get the foreground window's process ID
    fn get_foreground_pid_internal(&self) -> Option<u32> {
        unsafe {
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }

            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);

            if pid == 0 {
                None
            } else {
                Some(pid)
            }
        }
    }
}

impl Default for WindowsProcessEnumerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessEnumerator for WindowsProcessEnumerator {
    fn enumerate(&mut self) -> Result<Vec<ProcessInfo>> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot.is_null() || snapshot == (-1isize) as HANDLE {
                return Err(SmartFreezeError::ProcessEnumeration(
                    "Failed to create process snapshot".to_string(),
                ));
            }

            let mut processes = Vec::new();
            let mut entry: PROCESSENTRY32W = mem::zeroed();
            entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

            let foreground_pid = self.get_foreground_pid_internal();

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;
                    let parent_pid = entry.th32ParentProcessID;

                    // Store parent relationship
                    self.parent_map.insert(pid, parent_pid);
                    self.categorizer.update_parent_map(pid, parent_pid);

                    if pid != 0 {
                        let (name, full_path) = self.get_process_info(pid);
                        
                        if !name.is_empty() {
                            let memory_mb = self.get_memory_usage(pid);
                            let is_foreground = foreground_pid == Some(pid);
                            let category = self.categorizer.categorize(pid, &name, &full_path);

                            processes.push(ProcessInfo::new(
                                pid,
                                name,
                                full_path,
                                memory_mb,
                                is_foreground,
                                category,
                            ));
                        }
                    }

                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
            Ok(processes)
        }
    }

    fn get_foreground_pid(&self) -> Option<u32> {
        self.get_foreground_pid_internal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerator_creation() {
        let enumerator = WindowsProcessEnumerator::new();
        assert!(enumerator.parent_map.is_empty());
    }

    #[test]
    #[cfg(windows)]
    fn test_enumerate_processes() {
        let mut enumerator = WindowsProcessEnumerator::new();
        let result = enumerator.enumerate();
        
        assert!(result.is_ok());
        let processes = result.unwrap();
        assert!(!processes.is_empty()); // Should have at least some processes
        
        // Check that we have explorer.exe (should always be running)
        let has_explorer = processes.iter().any(|p| {
            p.name.eq_ignore_ascii_case("explorer.exe")
        });
        assert!(has_explorer, "Explorer.exe should be running");
    }

    #[test]
    #[cfg(windows)]
    fn test_foreground_pid() {
        let enumerator = WindowsProcessEnumerator::new();
        let foreground = enumerator.get_foreground_pid();
        
        // Should have a foreground process (this test itself)
        assert!(foreground.is_some());
    }
}
