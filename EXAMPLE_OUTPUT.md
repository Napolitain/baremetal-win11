# Example Output

This document shows example output from the smart freeze engine when run on a Windows system.

## Sample Run

```
Smart Freeze Engine - Process Monitor
======================================

Foreground Process ID: 12345

Finding heavy but safe-to-freeze processes (>100 MB)...

Found 5 processes safe to freeze:

PID      Name                                     Memory (MB) Category
----------------------------------------------------------------------------------
8234     chrome.exe                                       450 Productivity
9876     UbisoftConnect.exe                               320 Background
5432     nvcontainer.exe                                  280 Background
7654     JetBrains.Toolbox.exe                            210 Background
3210     spotify.exe                                      150 Productivity

Total memory usage: 1410 MB


All Running Processes:
======================

Total processes: 142

Top 10 by memory usage:
PID      Name                                Memory (MB) Category        Foreground
------------------------------------------------------------------------------------
8234     chrome.exe                                 450 Productivity
9876     steam.exe                                  380 Gaming
5432     discord.exe                                280 Communication
7654     UbisoftConnect.exe                         210 Background
3210     spotify.exe                                150 Productivity
12345    Code.exe                                   140 Productivity    YES
4567     msedge.exe                                 130 Productivity
2890     nvcontainer.exe                            125 Background
1234     explorer.exe                               115 Critical
6789     dwm.exe                                    110 Critical
```

## Explanation

### Foreground Process
- **PID 12345** (Code.exe) is the active application
- This is marked with "YES" in the Foreground column
- It will NOT be included in the list of processes safe to freeze

### Processes Safe to Freeze
The engine identified 5 applications using significant memory that can be safely suspended:
- **chrome.exe** - 450 MB (Productivity - browser with multiple tabs, background)
- **UbisoftConnect.exe** - 320 MB (Background Service - game launcher not in use)
- **nvcontainer.exe** - 280 MB (Background Service - NVIDIA helper process)
- **JetBrains.Toolbox.exe** - 210 MB (Background Service - IDE launcher)
- **spotify.exe** - 150 MB (Productivity - music player, paused)

Total potential memory savings: **1410 MB**

### Process Categories Explained
- **Gaming** (steam.exe, discord.exe during gaming): Protected from freezing to keep games responsive
- **Communication** (discord.exe): Potentially important, only frozen if using significant memory and not foreground
- **Background** (UbisoftConnect, nvcontainer): Safe to freeze - these are just launchers and services
- **Productivity** (chrome, spotify, Code.exe): Safe to freeze when not foreground
- **Critical** (explorer.exe, dwm.exe): Never frozen - essential system processes

### Protected Processes
Note that:
- **Critical system processes** like `explorer.exe` and `dwm.exe` are never suggested for freezing
- **Gaming processes** like `steam.exe` are protected to keep games responsive
- **Communication apps** like `discord.exe` are only frozen if using significant memory and not active

## Non-Windows Output

When run on Linux/macOS, the application shows:

```
Smart Freeze Engine - Process Monitor
======================================

WARNING: This application requires Windows to function.
The smart freeze engine uses Windows-specific APIs.

Please compile and run this on a Windows system.
```
