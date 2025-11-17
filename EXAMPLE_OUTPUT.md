# Example Output

This document shows example output from the smart freeze engine when run on a Windows system.

## Sample Run

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
9876     slack.exe                                                  320
5432     Teams.exe                                                  280
7654     Discord.exe                                                210
3210     spotify.exe                                                150
12345    Code.exe                                                   140 YES
4567     msedge.exe                                                 130
2890     Outlook.exe                                                125
1234     explorer.exe                                               115
6789     dwm.exe                                                    110
```

## Explanation

### Foreground Process
- **PID 12345** (Code.exe) is the active application
- This is marked with "YES" in the Foreground column
- It will NOT be included in the list of processes safe to freeze

### Processes Safe to Freeze
The engine identified 5 applications using significant memory that can be safely suspended:
- **chrome.exe** - 450 MB (multiple tabs, background)
- **slack.exe** - 320 MB (idle chat client)
- **Teams.exe** - 280 MB (not currently in use)
- **Discord.exe** - 210 MB (background chat)
- **spotify.exe** - 150 MB (music player, paused)

Total potential memory savings: **1410 MB**

### Protected Processes
Note that critical system processes like `explorer.exe` and `dwm.exe` are shown in the top 10 list but are never suggested for freezing, even if they meet the memory threshold.

## Non-Windows Output

When run on Linux/macOS, the application shows:

```
Smart Freeze Engine - Process Monitor
======================================

WARNING: This application requires Windows to function.
The smart freeze engine uses Windows-specific APIs.

Please compile and run this on a Windows system.
```
