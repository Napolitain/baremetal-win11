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

/// Process importance category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ProcessCategory {
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

/// Represents a process with its resource usage
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProcessInfo {
    pid: u32,
    name: String,
    full_path: String,
    memory_mb: u64,
    #[allow(dead_code)]
    cpu_percent: f64,
    is_foreground: bool,
    category: ProcessCategory,
}

/// Smart freeze engine that detects heavy but safe-to-freeze processes
#[allow(dead_code)]
struct SmartFreezeEngine {
    #[allow(dead_code)]
    previous_cpu_times: HashMap<u32, u64>,
    #[allow(dead_code)]
    processor_count: u32,
    /// Maps process PID to parent PID
    parent_map: HashMap<u32, u32>,
    /// Maps PID to process name for quick lookup
    process_names: HashMap<u32, String>,
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
            parent_map: HashMap::new(),
            process_names: HashMap::new(),
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

        // Clear previous maps
        self.parent_map.clear();
        self.process_names.clear();

        unsafe {
            // Create snapshot of all processes
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot.is_null() || snapshot == (-1isize) as HANDLE {
                return processes;
            }

            let mut entry: PROCESSENTRY32W = mem::zeroed();
            entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

            // First pass: build parent map
            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;
                    let parent_pid = entry.th32ParentProcessID;

                    // Store parent relationship
                    if pid != 0 {
                        self.parent_map.insert(pid, parent_pid);
                    }

                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);

            // Second pass: get detailed process info
            let snapshot2 = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot2.is_null() || snapshot2 == (-1isize) as HANDLE {
                return processes;
            }

            entry = mem::zeroed();
            entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot2, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;

                    // Get process info
                    if let Some(process_info) = self.get_process_info(pid, foreground_pid) {
                        // Store process name for parent lookup
                        self.process_names.insert(pid, process_info.name.clone());
                        processes.push(process_info);
                    }

                    if Process32NextW(snapshot2, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot2);
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

            // Get process name and full path
            let (name, full_path) = self
                .get_process_name_and_path(process)
                .unwrap_or_else(|| (format!("PID_{}", pid), String::new()));

            // Get memory usage
            let memory_mb = self.get_memory_usage(process);

            // Check if this is the foreground process
            let is_foreground = foreground_pid.map_or(false, |fg_pid| fg_pid == pid);

            // Categorize the process using name, path, and parent info
            let category = self.categorize_process(pid, &name, &full_path);

            CloseHandle(process);

            Some(ProcessInfo {
                pid,
                name,
                full_path,
                memory_mb,
                cpu_percent: 0.0, // CPU calculation requires time between samples
                is_foreground,
                category,
            })
        }
    }

    /// Get process name and full path from handle
    fn get_process_name_and_path(&self, process: HANDLE) -> Option<(String, String)> {
        unsafe {
            let mut buffer = vec![0u16; 1024];
            let mut size = buffer.len() as u32;

            // QueryFullProcessImageNameW with flags = 0 for native path
            if QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size) != 0 {
                // Convert to String
                let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
                // Extract just the filename
                let name = full_path
                    .split('\\')
                    .last()
                    .unwrap_or(&full_path)
                    .to_string();
                Some((name, full_path))
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
    /// - Not gaming-related processes (to keep games responsive)
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
                // 3. Not critical system processes
                && p.category != ProcessCategory::Critical
                // 4. Not gaming processes (important to keep responsive)
                && p.category != ProcessCategory::Gaming
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

    /// Categorize a process based on its name, path, and parent process
    ///
    /// Strategies used:
    /// 1. **Pattern matching on executable names** - Most reliable for known applications
    /// 2. **Parent process detection** - Games often launched by Steam/Epic/etc
    /// 3. **Path analysis** - Games typically in specific directories (Steam, Epic, etc.)
    /// 4. **Resource usage patterns** (future) - Games use GPU, browsers use many processes
    fn categorize_process(&self, pid: u32, name: &str, full_path: &str) -> ProcessCategory {
        let name_lower = name.to_lowercase();
        let path_lower = full_path.to_lowercase();

        // Critical system processes
        if self.is_critical_process(name) {
            return ProcessCategory::Critical;
        }

        // Strategy 3: Path Analysis - Check if running from gaming directories
        // This is very important as it catches games we don't know by name
        let gaming_path_patterns = [
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
            "\\program files\\steam\\",
            "\\program files (x86)\\steam\\",
            // Common custom Steam library locations
            "\\games\\",
            "\\my games\\",
        ];

        for pattern in &gaming_path_patterns {
            if path_lower.contains(pattern) {
                // Additional check: not a launcher itself running from these dirs
                if !name_lower.contains("launcher")
                    && !name_lower.contains("update")
                    && !name_lower.contains("crash")
                {
                    return ProcessCategory::Gaming;
                }
            }
        }

        // Strategy 2: Parent Process Detection
        // Check if parent is a gaming launcher
        if let Some(&parent_pid) = self.parent_map.get(&pid) {
            if let Some(parent_name) = self.process_names.get(&parent_pid) {
                let parent_lower = parent_name.to_lowercase();
                let gaming_parent_patterns = [
                    "steam.exe",
                    "epicgameslauncher.exe",
                    "origin.exe",
                    "battle.net.exe",
                    "gog.exe",
                    "galaxyclient.exe",
                ];

                for pattern in &gaming_parent_patterns {
                    if parent_lower.contains(pattern) {
                        // If launched by a game launcher, it's likely a game
                        return ProcessCategory::Gaming;
                    }
                }
            }
        }

        // Gaming category - Important to keep responsive
        // Strategy: Match game launchers, game executables, and anti-cheat services
        let gaming_patterns = [
            // Game launchers
            "steam.exe",
            "steamwebhelper.exe",
            "steamservice.exe",
            "epicgameslauncher.exe",
            "epicwebhelper.exe",
            "origin.exe",
            "originwebhelper.exe",
            "gog.exe",
            "galaxyclient.exe",
            "battle.net.exe",
            "blizzard.exe",
            // Game executables (common patterns)
            "game.exe",
            "launcher.exe",
            // Anti-cheat systems
            "easyanticheat.exe",
            "battleye.exe",
            "vanguard.exe",
            // Common game executables (examples)
            "csgo.exe",
            "dota2.exe",
            "leagueoflegends.exe",
            "valorant.exe",
            "overwatch.exe",
            "minecraft.exe",
        ];

        for pattern in &gaming_patterns {
            if name_lower.contains(pattern) {
                return ProcessCategory::Gaming;
            }
        }

        // Communication category - Potentially important
        // Strategy: Match known chat/voice/video apps
        let communication_patterns = [
            "discord.exe",
            "discordcanary.exe",
            "discordptb.exe",
            "slack.exe",
            "teams.exe",
            "zoom.exe",
            "skype.exe",
            "telegram.exe",
            "signal.exe",
            "element.exe",
            "matrix.exe",
            "riot.exe",
            "mumble.exe",
            "teamspeak.exe",
            "ventrilo.exe",
            "whatsapp.exe",
            "messenger.exe",
        ];

        for pattern in &communication_patterns {
            if name_lower.contains(pattern) {
                return ProcessCategory::Communication;
            }
        }

        // Background services and launchers - Safe to freeze
        // Strategy: Match known background services, updaters, and launchers
        let background_patterns = [
            // Launchers that are just background services
            "uplay.exe",
            "upc.exe",
            "ubisoftconnect.exe",
            "epiconlineservices.exe",
            // Graphics/hardware utilities
            "nvcontainer.exe",
            "nvidia share.exe",
            "geforce experience.exe",
            "nvcplui.exe",
            "nvprofileupdater.exe",
            "amdrsserv.exe",
            "radeonsoft.exe",
            // Developer tools and IDEs (when not foreground)
            "jetbrains.toolbox.exe",
            "jetbrains-toolbox.exe",
            // Updaters and helpers
            "update.exe",
            "updater.exe",
            "helper.exe",
            "crashhandler.exe",
            "crashreporter.exe",
            // Cloud sync services
            "onedrive.exe",
            "dropbox.exe",
            "googledrivesync.exe",
        ];

        for pattern in &background_patterns {
            if name_lower.contains(pattern) {
                return ProcessCategory::BackgroundService;
            }
        }

        // Productivity/Browsers - Safe to freeze when not foreground
        // Strategy: Match browsers, office apps, media players
        let productivity_patterns = [
            // Browsers
            "chrome.exe",
            "firefox.exe",
            "msedge.exe",
            "edge.exe",
            "opera.exe",
            "brave.exe",
            "vivaldi.exe",
            // Office/Productivity
            "excel.exe",
            "word.exe",
            "powerpoint.exe",
            "outlook.exe",
            "onenote.exe",
            "notion.exe",
            "obsidian.exe",
            // IDEs and editors (when not foreground)
            "code.exe",
            "code-insiders.exe",
            "atom.exe",
            "sublime.exe",
            "notepad++.exe",
            "pycharm.exe",
            "intellij.exe",
            "rider.exe",
            // Media players
            "spotify.exe",
            "vlc.exe",
            "itunes.exe",
            "musicbee.exe",
        ];

        for pattern in &productivity_patterns {
            if name_lower.contains(pattern) {
                return ProcessCategory::Productivity;
            }
        }

        // Default to unknown
        ProcessCategory::Unknown
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
            parent_map: HashMap::new(),
            process_names: HashMap::new(),
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

    fn get_process_name_and_path(&self, _process: usize) -> Option<(String, String)> {
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

    fn categorize_process(&self, _pid: u32, _name: &str, _full_path: &str) -> ProcessCategory {
        ProcessCategory::Unknown
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
        println!(
            "{:<8} {:<40} {:>12} {:<20}",
            "PID", "Name", "Memory (MB)", "Category"
        );
        println!("{}", "-".repeat(82));

        for process in &heavy_processes {
            let category_str = match process.category {
                ProcessCategory::Critical => "Critical",
                ProcessCategory::Gaming => "Gaming",
                ProcessCategory::Communication => "Communication",
                ProcessCategory::BackgroundService => "Background",
                ProcessCategory::Productivity => "Productivity",
                ProcessCategory::Unknown => "Unknown",
            };
            println!(
                "{:<8} {:<40} {:>12} {:<20}",
                process.pid, process.name, process.memory_mb, category_str
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
        "{:<8} {:<35} {:>12} {:<15} {:<12}",
        "PID", "Name", "Memory (MB)", "Category", "Foreground"
    );
    println!("{}", "-".repeat(84));

    for process in sorted.iter().take(10) {
        let category_str = match process.category {
            ProcessCategory::Critical => "Critical",
            ProcessCategory::Gaming => "Gaming",
            ProcessCategory::Communication => "Communication",
            ProcessCategory::BackgroundService => "Background",
            ProcessCategory::Productivity => "Productivity",
            ProcessCategory::Unknown => "Unknown",
        };
        println!(
            "{:<8} {:<35} {:>12} {:<15} {:<12}",
            process.pid,
            process.name,
            process.memory_mb,
            category_str,
            if process.is_foreground { "YES" } else { "" }
        );
    }
}
