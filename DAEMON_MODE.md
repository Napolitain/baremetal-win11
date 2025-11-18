# Daemon Mode - Automatic Game Performance Optimization

SmartFreeze v0.3.0 introduces **Daemon Mode** - a background service that automatically freezes heavy processes when you launch a game, then resumes them when you're done gaming.

## Features

### 🎮 Automatic Game Detection
- Continuously monitors for gaming processes (every 60 seconds by default)
- Uses intelligent categorization to detect games from Steam, Epic, GOG, etc.
- Detects games via path analysis, launcher detection, and known patterns

### ⚡ Smart Freezing
- Automatically freezes background processes when a game is detected
- Respects critical system processes (never freezes TextInputHost, SearchHost, etc.)
- Protects foreground applications and communication apps
- Configurable memory threshold (default: 100 MB)

### 🔄 Automatic Resume
- Resumes all frozen processes when you exit the game
- Tracks frozen PIDs to ensure correct resumption
- No manual intervention required

### 🖥️ System Tray Integration
- Runs minimized in the system tray
- Right-click menu for quick access:
  - **Enable/Disable Auto-Freeze**: Toggle automatic freezing on/off
  - **Run on Windows Startup**: Add/remove from startup registry
  - **Quit**: Exit daemon and resume all frozen processes

## Usage

### Start Daemon Mode

```bash
# Run with default settings (60 second check interval, 100 MB threshold)
smart-freeze.exe --daemon

# Run with custom check interval (30 seconds)
smart-freeze.exe --daemon --interval 30

# Run with custom memory threshold (200 MB)
smart-freeze.exe --daemon --threshold 200
```

### Windows Startup Integration

```bash
# Add to Windows startup (runs automatically on boot)
smart-freeze.exe --install-startup

# Remove from Windows startup
smart-freeze.exe --uninstall-startup
```

When installed to startup, the daemon will automatically run with:
- Default 60 second check interval
- 100 MB memory threshold
- System tray icon enabled

### System Tray Controls

Once running in daemon mode, look for the SmartFreeze icon in your system tray (blue square icon).

**Right-click the icon to access:**
- **Enable Auto-Freeze**: Toggle automatic process freezing (enabled by default)
- **Run on Windows Startup**: Install/uninstall startup registry entry
- **Quit**: Exit the daemon and resume all frozen processes

## How It Works

### Detection Loop

Every 60 seconds (configurable), the daemon:
1. Scans all running processes
2. Checks if any Gaming category processes are active
3. If game detected:
   - Gets list of safe-to-freeze processes (based on threshold)
   - Freezes all background services, browsers, productivity apps
   - Logs actions to console
4. If no game detected and processes were previously frozen:
   - Resumes all frozen processes
   - Logs actions to console

### Protected Processes

The following are **never frozen**:
- **Critical System**: explorer.exe, dwm.exe, svchost.exe, textinputhost.exe, searchhost.exe, startmenuexperiencehost.exe
- **Gaming Processes**: Detected games and game launchers
- **Foreground Applications**: Any app currently in focus
- **Communication Apps** (optional): Discord, Slack, Teams, etc. (marked as "potentially important")

### Safe-to-Freeze Categories

The daemon will freeze:
- **Browsers**: Chrome, Firefox, Edge, Brave (when not in foreground)
- **Background Services**: NVIDIA GeForce Experience, Google Drive, OneDrive, Intel utilities
- **Productivity Apps**: VSCode, Office apps, Spotify (when not in foreground)
- **Developer Tools**: JetBrains Toolbox, Docker Desktop

## Performance Impact

### Resource Usage (Idle)
- **Memory**: ~5-10 MB
- **CPU**: <0.1%
- **Disk**: None (except startup registry entry)

### Check Interval Recommendations
- **60 seconds** (default): Best for battery life, minimal overhead
- **30 seconds**: Faster game detection, slight CPU increase
- **120 seconds**: Ultra-low overhead for laptops on battery

## Configuration

Currently, configuration is via command-line arguments. Future versions will support a config file.

### Recommended Settings

**Desktop Gaming PC:**
```bash
smart-freeze.exe --daemon --interval 60 --threshold 100
```

**Laptop (Battery Conscious):**
```bash
smart-freeze.exe --daemon --interval 120 --threshold 150
```

**High-RAM System (Aggressive Freezing):**
```bash
smart-freeze.exe --daemon --interval 30 --threshold 50
```

## Troubleshooting

### Daemon doesn't start
- Ensure you're running on Windows 10/11
- Check if another instance is already running
- Run from command prompt to see error messages

### Games not detected
- Verify game path matches known patterns (Steam, Epic, etc.)
- Check verbose output with `--verbose` flag
- Report missing games as GitHub issues

### Processes not resuming
- Use system tray "Quit" option (automatically resumes all)
- Manually resume with: `smart-freeze.exe --action resume --pid <PID>`

### System tray icon not appearing
- Check Windows taskbar settings (ensure system tray icons are visible)
- Look in "hidden icons" overflow menu
- Restart daemon if icon disappears

## Logging

Daemon mode logs to console/stdout:
```
[SmartFreeze] Game detected! Freezing background processes...
[SmartFreeze] Froze chrome.exe (PID 1234)
[SmartFreeze] Froze jetbrains-toolbox.exe (PID 5678)
...
[SmartFreeze] No game detected. Resuming processes...
[SmartFreeze] Resumed PID 1234
[SmartFreeze] Resumed PID 5678
```

To capture logs:
```bash
smart-freeze.exe --daemon > smartfreeze.log 2>&1
```

## Security & Privacy

- **No Network Access**: Daemon runs entirely offline
- **Local Registry Only**: Only modifies `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
- **No Telemetry**: No data collection or reporting
- **Open Source**: All code available for audit at [GitHub](https://github.com/Napolitain/baremetal-win11)

## Future Enhancements

- [ ] Configuration file support (TOML/JSON)
- [ ] Custom process whitelist/blacklist
- [ ] Per-game profiles (different thresholds for different games)
- [ ] System event-based detection (instead of polling)
- [ ] GPU usage detection for better game identification
- [ ] Toast notifications when freezing/resuming
- [ ] Web UI for configuration
- [ ] Performance metrics and statistics

## Comparison: Manual vs Daemon Mode

| Feature | Manual Mode | Daemon Mode |
|---------|-------------|-------------|
| Game Detection | Manual | Automatic |
| Process Freezing | Manual command | Automatic |
| Process Resuming | Manual command | Automatic |
| System Tray | ❌ | ✅ |
| Startup Integration | ❌ | ✅ |
| Set & Forget | ❌ | ✅ |
| Resource Usage | None (when not running) | ~5-10 MB RAM |
| Check Interval | N/A | Configurable (60s default) |

## Example Workflow

1. **Initial Setup** (one-time):
   ```bash
   smart-freeze.exe --install-startup
   ```

2. **Start Gaming**:
   - Launch Steam/Epic/GOG and start your game
   - Daemon detects game within 60 seconds
   - Automatically freezes heavy background processes
   - You get more performance and smoother gameplay

3. **Exit Game**:
   - Close your game
   - Daemon detects game exit within 60 seconds
   - Automatically resumes all frozen processes
   - System returns to normal

4. **Disable Temporarily**:
   - Right-click system tray icon
   - Uncheck "Enable Auto-Freeze"
   - Daemon continues running but won't freeze processes

## Credits

Daemon mode uses:
- [tray-icon](https://crates.io/crates/tray-icon) - Cross-platform system tray
- [winit](https://crates.io/crates/winit) - Window event loop
- [windows-sys](https://crates.io/crates/windows-sys) - Windows API bindings
