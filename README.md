# baremetal-win11

A smart freeze engine for Windows 11 that intelligently identifies heavy but safe-to-freeze applications to optimize system performance.

## Features

- **Process Enumeration**: Lists all running processes on the system
- **CPU & Memory Detection**: Monitors CPU and memory usage of processes
- **Foreground Window Detection**: Identifies the currently active application to avoid freezing it
- **Intelligent Process Categorization**: Automatically categorizes processes into importance levels
- **Smart Filtering**: Returns a list of heavy processes that are safe to freeze based on:
  - Memory usage threshold (configurable, default: 100 MB)
  - Foreground status (active apps are never frozen)
  - Process category (critical and gaming processes protected by default)

## Architecture

The smart freeze engine uses Win32 APIs to:
- Enumerate processes via `CreateToolhelp32Snapshot` and `Process32FirstW/NextW`
- Query process memory usage with `GetProcessMemoryInfo`
- Detect the foreground window using `GetForegroundWindow` and `GetWindowThreadProcessId`
- Query process names with `QueryFullProcessImageNameW`

### Process Categorization Strategies

The engine uses multiple strategies to intelligently categorize processes:

#### 1. **Critical System Processes** (Never Freeze)
Protected system processes essential for Windows operation:
- System, smss.exe, csrss.exe, wininit.exe, services.exe
- lsass.exe, svchost.exe, winlogon.exe
- explorer.exe, dwm.exe (Desktop Window Manager)

#### 2. **Gaming Processes** (Important to Keep)
Game launchers, executables, and related services that should remain responsive:
- **Game Launchers**: Steam, Epic Games, Origin, GOG Galaxy, Battle.net
- **Anti-cheat Systems**: EasyAntiCheat, BattlEye, Vanguard
- **Game Executables**: Common patterns like game.exe, launcher.exe
- **Examples**: CS:GO, Dota 2, League of Legends, Valorant, Overwatch, Minecraft

**Strategy**: Pattern matching on executable names for known game-related processes

#### 3. **Communication Apps** (Potentially Important)
Chat, voice, and video applications that users may want to keep responsive:
- **Chat Apps**: Discord, Slack, Teams, Telegram, Signal, WhatsApp
- **Voice/Video**: Zoom, Skype, Mumble, TeamSpeak, Ventrilo
- **Matrix Clients**: Element, Riot

**Strategy**: Match known communication app executable names

#### 4. **Background Services** (Safe to Freeze)
Launchers, updaters, and utilities that run in the background:
- **Game Launchers (Background)**: Ubisoft Connect, Epic Online Services
- **Graphics Utilities**: NVIDIA GeForce Experience, AMD Radeon Software
- **Developer Tools**: JetBrains Toolbox
- **Updaters**: Various update.exe, updater.exe, helper.exe processes
- **Cloud Sync**: OneDrive, Dropbox, Google Drive Sync

**Strategy**: Match known background service patterns and utilities

#### 5. **Productivity/Browsers** (Safe to Freeze When Not Foreground)
Applications that are safe to suspend when not actively in use:
- **Browsers**: Chrome, Firefox, Edge, Opera, Brave, Vivaldi
- **Office Apps**: Excel, Word, PowerPoint, Outlook, OneNote
- **IDEs/Editors**: VSCode, PyCharm, IntelliJ, Rider, Sublime Text, Atom
- **Media Players**: Spotify, VLC, iTunes, MusicBee
- **Note-taking**: Notion, Obsidian

**Strategy**: Match productivity tool patterns, safe to freeze when not foreground

### Future Categorization Strategies

Additional strategies that can be implemented:
- **Parent Process Detection**: Games launched by Steam/Epic have specific parent processes
- **Path Analysis**: Games typically installed in `C:\Program Files\` or `C:\Games\`
- **Resource Usage Patterns**: Games typically use GPU, browsers spawn many child processes
- **Process Tree Analysis**: Identify related processes (e.g., browser with multiple helpers)

## Requirements

- **Operating System**: Windows 11 (or Windows 10)
- **Rust**: 1.70 or later
- **Cargo**: Latest stable version

## Building

```bash
# Clone the repository
git clone https://github.com/Napolitain/baremetal-win11.git
cd baremetal-win11

# Build the project
cargo build --release

# Run the application
cargo run --release
```

**Note**: This application **must be compiled and run on Windows** as it uses Windows-specific APIs. Building on Linux/macOS will compile successfully but will display a warning message when run.

## Usage

Simply run the compiled binary:

```bash
./target/release/smart-freeze.exe
```

The application will:
1. Display the current foreground process ID
2. List all processes using more than 100 MB of memory that are safe to freeze
3. Show the top 10 processes by memory usage for reference

### Example Output

```
Smart Freeze Engine - Process Monitor
======================================

Foreground Process ID: 12345

Finding heavy but safe-to-freeze processes (>100 MB)...

Found 5 processes safe to freeze:

PID      Name                                               Memory (MB)
------------------------------------------------------------------------
8234     chrome.exe                                                 450
9876     slack.exe                                                  320
5432     Teams.exe                                                  280
7654     Discord.exe                                                210
3210     spotify.exe                                                150

Total memory usage: 1410 MB

All Running Processes:
======================

Total processes: 142

Top 10 by memory usage:
PID      Name                                               Memory (MB) Foreground
------------------------------------------------------------------------------------
8234     chrome.exe                                                 450
...
```

## Future Enhancements

### GPU Usage Detection (Optional)

GPU usage detection requires additional FFI bindings to:
- **DXGI** (DirectX Graphics Infrastructure) - for DirectX applications
- **NVAPI** (NVIDIA API) - for NVIDIA GPU monitoring
- **ADL** (AMD Display Library) - for AMD GPU monitoring

These features are intentionally omitted for the initial prototype to keep the implementation simple and focused on CPU + memory + foreground detection.

### CPU Usage Tracking

The current implementation includes a framework for CPU usage tracking (`cpu_percent` field), but actual CPU usage calculation requires:
- Tracking process CPU time over intervals
- Multiple sampling periods for accuracy
- Additional Win32 API calls (`GetProcessTimes`)

This can be added in future iterations.

## Development

### Project Structure

```
baremetal-win11/
├── Cargo.toml          # Project dependencies
├── README.md           # This file
└── src/
    └── main.rs         # Smart freeze engine implementation
```

### Dependencies

- `windows-sys`: Provides low-level bindings to Win32 APIs (Windows only)

### Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

See the [LICENSE](LICENSE) file for details.

## Technical Details

### Memory Detection

Memory usage is determined by reading the `WorkingSetSize` from the process's `PROCESS_MEMORY_COUNTERS` structure, which represents the amount of physical RAM currently used by the process.

### Foreground Detection

The foreground window is detected by calling `GetForegroundWindow()` to get the currently active window handle, then `GetWindowThreadProcessId()` to retrieve the process ID that owns that window.

### Process Enumeration

Process enumeration uses the Toolhelp API:
1. `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)` creates a snapshot
2. `Process32FirstW()` and `Process32NextW()` iterate through processes
3. Each process entry contains the PID and basic information

### Safety Considerations

- **System Stability**: The engine never suggests freezing critical system processes
- **User Experience**: Active foreground applications are always protected
- **Resource Management**: All Win32 handles are properly closed to prevent leaks

## Platform Support

| Platform | Build | Run |
|----------|-------|-----|
| Windows 11 | ✅ | ✅ |
| Windows 10 | ✅ | ✅ |
| Linux | ✅ | ⚠️ (warning only) |
| macOS | ✅ | ⚠️ (warning only) |

The application compiles on all platforms but only functions on Windows.