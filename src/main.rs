use std::collections::HashMap;

#[cfg(windows)]
use std::mem;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
#[cfg(windows)]
use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Represents a process with its resource usage
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProcessInfo {
    pid: u32,
    name: String,
    memory_mb: u64,
    #[allow(dead_code)]
    cpu_percent: f64,
    is_foreground: bool,
}

/// Smart freeze engine that detects heavy but safe-to-freeze processes
#[allow(dead_code)]
struct SmartFreezeEngine {
    #[allow(dead_code)]
    previous_cpu_times: HashMap<u32, u64>,
    #[allow(dead_code)]
    processor_count: u32,
}

#[cfg(windows)]
impl SmartFreezeEngine {
    fn new() -> Self {
        let processor_count = unsafe {
            let mut sys_info: SYSTEM_INFO = mem::zeroed();
            GetSystemInfo(&mut sys_info);
            sys_info.dwNumberOfProcessors
        };

        Self {
            previous_cpu_times: HashMap::new(),
            processor_count,
        }
    }

    /// Get the foreground window's process ID
    fn get_foreground_pid() -> Option<u32> {
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

    /// Enumerate all running processes
    fn enumerate_processes(&mut self) -> Vec<ProcessInfo> {
        let foreground_pid = Self::get_foreground_pid();
        let mut processes = Vec::new();

        unsafe {
            // Create snapshot of all processes
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot.is_null() || snapshot == (-1isize) as HANDLE {
                return processes;
            }

            let mut entry: PROCESSENTRY32W = mem::zeroed();
            entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

            // Iterate through processes
            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;

                    // Get process info
                    if let Some(process_info) = self.get_process_info(pid, foreground_pid) {
                        processes.push(process_info);
                    }

                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
        }

        processes
    }

    /// Get detailed information about a specific process
    fn get_process_info(&self, pid: u32, foreground_pid: Option<u32>) -> Option<ProcessInfo> {
        if pid == 0 {
            return None;
        }

        unsafe {
            // Open process with query permissions
            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
            if process.is_null() {
                return None;
            }

            // Get process name
            let name = self
                .get_process_name(process)
                .unwrap_or_else(|| format!("PID_{}", pid));

            // Get memory usage
            let memory_mb = self.get_memory_usage(process);

            // Check if this is the foreground process
            let is_foreground = foreground_pid.map_or(false, |fg_pid| fg_pid == pid);

            CloseHandle(process);

            Some(ProcessInfo {
                pid,
                name,
                memory_mb,
                cpu_percent: 0.0, // CPU calculation requires time between samples
                is_foreground,
            })
        }
    }

    /// Get process name from handle
    fn get_process_name(&self, process: HANDLE) -> Option<String> {
        unsafe {
            let mut buffer = vec![0u16; 1024];
            let mut size = buffer.len() as u32;

            // QueryFullProcessImageNameW with flags = 0 for native path
            if QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size) != 0 {
                // Convert to String and extract just the filename
                let path = String::from_utf16_lossy(&buffer[..size as usize]);
                path.split('\\').last().map(|s| s.to_string())
            } else {
                None
            }
        }
    }

    /// Get memory usage in MB for a process
    fn get_memory_usage(&self, process: HANDLE) -> u64 {
        unsafe {
            let mut pmc: PROCESS_MEMORY_COUNTERS = mem::zeroed();
            pmc.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

            if GetProcessMemoryInfo(process, &mut pmc, pmc.cb) != 0 {
                pmc.WorkingSetSize as u64 / 1024 / 1024
            } else {
                0
            }
        }
    }

    /// Find heavy but safe-to-freeze processes
    /// Criteria:
    /// - Memory usage > 100 MB (configurable threshold)
    /// - Not the foreground process
    /// - Not critical system processes
    fn find_heavy_processes(&mut self, min_memory_mb: u64) -> Vec<ProcessInfo> {
        let processes = self.enumerate_processes();

        processes
            .into_iter()
            .filter(|p| {
                // Filter criteria:
                // 1. Using significant memory
                p.memory_mb >= min_memory_mb
                // 2. Not the foreground process
                && !p.is_foreground
                // 3. Not critical system processes (simple check by name)
                && !self.is_critical_process(&p.name)
            })
            .collect()
    }

    /// Check if a process is critical (should never be frozen)
    fn is_critical_process(&self, name: &str) -> bool {
        let critical = [
            "System",
            "smss.exe",
            "csrss.exe",
            "wininit.exe",
            "services.exe",
            "lsass.exe",
            "svchost.exe",
            "winlogon.exe",
            "explorer.exe",
            "dwm.exe",
        ];

        critical.iter().any(|&c| name.eq_ignore_ascii_case(c))
    }
}

// Non-Windows stub implementation
#[cfg(not(windows))]
#[allow(dead_code)]
impl SmartFreezeEngine {
    fn new() -> Self {
        Self {
            previous_cpu_times: HashMap::new(),
            processor_count: 0,
        }
    }

    fn get_foreground_pid() -> Option<u32> {
        None
    }

    fn enumerate_processes(&mut self) -> Vec<ProcessInfo> {
        Vec::new()
    }

    fn get_process_info(&self, _pid: u32, _foreground_pid: Option<u32>) -> Option<ProcessInfo> {
        None
    }

    fn get_process_name(&self, _process: usize) -> Option<String> {
        None
    }

    fn get_memory_usage(&self, _process: usize) -> u64 {
        0
    }

    fn find_heavy_processes(&mut self, _min_memory_mb: u64) -> Vec<ProcessInfo> {
        Vec::new()
    }

    fn is_critical_process(&self, _name: &str) -> bool {
        false
    }
}

fn main() {
    println!("Smart Freeze Engine - Process Monitor");
    println!("======================================\n");

    #[cfg(not(windows))]
    {
        println!("WARNING: This application requires Windows to function.");
        println!("The smart freeze engine uses Windows-specific APIs.");
        println!("\nPlease compile and run this on a Windows system.");
    }

    #[cfg(windows)]
    {
        run_engine();
    }
}

#[cfg(windows)]
fn run_engine() {
    let mut engine = SmartFreezeEngine::new();

    // Get foreground process
    if let Some(fg_pid) = SmartFreezeEngine::get_foreground_pid() {
        println!("Foreground Process ID: {}", fg_pid);
    } else {
        println!("No foreground process detected");
    }
    println!();

    // Find heavy processes (using >100 MB as threshold)
    println!("Finding heavy but safe-to-freeze processes (>100 MB)...\n");
    let heavy_processes = engine.find_heavy_processes(100);

    if heavy_processes.is_empty() {
        println!("No heavy processes found that are safe to freeze.");
    } else {
        println!(
            "Found {} processes safe to freeze:\n",
            heavy_processes.len()
        );
        println!("{:<8} {:<50} {:>12}", "PID", "Name", "Memory (MB)");
        println!("{}", "-".repeat(72));

        for process in &heavy_processes {
            println!(
                "{:<8} {:<50} {:>12}",
                process.pid, process.name, process.memory_mb
            );
        }

        println!(
            "\nTotal memory usage: {} MB",
            heavy_processes.iter().map(|p| p.memory_mb).sum::<u64>()
        );
    }

    // Enumerate all processes for debugging
    println!("\n\nAll Running Processes:");
    println!("======================\n");
    let all_processes = engine.enumerate_processes();
    println!("Total processes: {}", all_processes.len());

    // Show top 10 by memory
    let mut sorted = all_processes.clone();
    sorted.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));

    println!("\nTop 10 by memory usage:");
    println!(
        "{:<8} {:<50} {:>12} {:<12}",
        "PID", "Name", "Memory (MB)", "Foreground"
    );
    println!("{}", "-".repeat(84));

    for process in sorted.iter().take(10) {
        println!(
            "{:<8} {:<50} {:>12} {:<12}",
            process.pid,
            process.name,
            process.memory_mb,
            if process.is_foreground { "YES" } else { "" }
        );
    }
}
