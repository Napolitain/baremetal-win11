# SmartFreeze - Quick Start Guide

## What is SmartFreeze?

SmartFreeze is an intelligent Windows process optimizer that automatically freezes heavy background processes when you're gaming, giving you better performance. It then resumes everything when you're done.

## 🚀 Quick Setup (30 seconds)

### Option 1: Auto-Start with Windows (Recommended)

1. Open PowerShell/Terminal in the SmartFreeze directory
2. Run: `.\target\release\smart-freeze.exe --install-startup`
3. Done! It will auto-start on next boot

### Option 2: Manual Start

1. Open PowerShell/Terminal
2. Run: `.\target\release\smart-freeze.exe --daemon`
3. Look for system tray icon (blue square)

## 🎮 How to Use

Once installed:

1. **Launch a game** (Steam, Epic, GOG, etc.)
2. **SmartFreeze detects it automatically** (within 60 seconds)
3. **Background processes freeze** (browsers, utilities, etc.)
4. **Game performance improves** ⚡
5. **Exit game**
6. **Processes resume automatically** ✅

**You don't have to do anything!**

## ⚙️ System Tray Controls

Right-click the SmartFreeze tray icon:

- **Enable Auto-Freeze**: Toggle on/off (enabled by default)
- **Run on Windows Startup**: Add/remove from startup
- **Quit**: Exit and resume all processes

## 📊 What Gets Frozen?

### ✅ Safe to Freeze
- Browsers (Chrome, Firefox, Edge, Brave)
- Background services (Google Drive, OneDrive, NVIDIA GeForce Experience)
- Productivity apps (Office, VSCode, Spotify)
- Developer tools (JetBrains Toolbox)

### ❌ Never Frozen
- Critical Windows processes (Explorer, DWM, System services)
- Your game and game launchers
- Active foreground applications
- Keyboard/search/Start menu processes

## 🔧 Advanced Usage

### Custom Check Interval

```bash
# Check every 30 seconds (faster detection)
smart-freeze.exe --daemon --interval 30

# Check every 120 seconds (lower resource use)
smart-freeze.exe --daemon --interval 120
```

### Custom Memory Threshold

```bash
# Only freeze processes using >200 MB
smart-freeze.exe --daemon --threshold 200

# Freeze more aggressively (>50 MB)
smart-freeze.exe --daemon --threshold 50
```

### Manual Mode (Without Daemon)

```bash
# List safe-to-freeze processes
smart-freeze.exe

# Freeze a specific process
smart-freeze.exe --action freeze --pid 1234

# Resume a frozen process
smart-freeze.exe --action resume --pid 1234
```

## 🛠️ Troubleshooting

### Daemon won't start
- Make sure you're on Windows 10 or 11
- Check if it's already running (look for tray icon)
- Run from Command Prompt to see error messages

### Game not detected
- Wait 60 seconds after launching (default check interval)
- Make sure game is from known launcher (Steam/Epic/GOG)
- Use `--interval 30` for faster detection

### Processes not resuming
- Use tray icon "Quit" button (auto-resumes all)
- Or manually: `smart-freeze.exe --action resume --pid <PID>`

### Tray icon missing
- Check hidden icons in taskbar overflow menu
- Restart daemon if needed

## 📖 Learn More

- **Full Documentation**: [README.md](README.md)
- **Daemon Mode Details**: [DAEMON_MODE.md](DAEMON_MODE.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)

## 🤝 Support

- **Issues**: [GitHub Issues](https://github.com/Napolitain/baremetal-win11/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Napolitain/baremetal-win11/discussions)

## 🎯 Performance Tips

1. **Use 60 second interval** - Best balance of responsiveness and resource use
2. **Keep threshold at 100 MB** - Catches heavy processes without over-freezing
3. **Enable Windows startup** - Set-and-forget convenience
4. **Don't freeze Discord** - It's already categorized as communication (optional freeze)

## 📈 Expected Results

Typical improvements when gaming:
- **2-5 GB RAM** freed up
- **5-15% CPU** freed up
- **Smoother gameplay** with less background interference
- **Better frame times** during intensive scenes

Results vary based on your background processes.

---

**Enjoy lag-free gaming! 🎮**
