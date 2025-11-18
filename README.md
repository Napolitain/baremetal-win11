# SmartFreeze

[![Build Status](https://github.com/Napolitain/baremetal-win11/actions/workflows/build.yml/badge.svg)](https://github.com/Napolitain/baremetal-win11/actions/workflows/build.yml)
[![Security Audit](https://github.com/Napolitain/baremetal-win11/actions/workflows/security-audit.yml/badge.svg)](https://github.com/Napolitain/baremetal-win11/actions/workflows/security-audit.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A smart process freezer for Windows that intelligently suspends heavy background applications during gaming to optimize performance.

## Features

- **Intelligent Process Detection**: Automatically categorizes processes by importance (critical, gaming, communication, background, productivity)
- **Daemon Mode**: Background service with system tray that auto-freezes processes when gaming
- **Crash Recovery**: Persistent state ensures frozen processes are resumed even after crashes
- **Gaming Detection**: Recognizes major launchers (Steam, Epic, GOG, Origin, Battle.net) and games
- **Communication Protection**: Optional flag to keep Discord, Teams, Slack running
- **Multiple Output Formats**: Table, JSON, CSV support
- **Manual Control**: Freeze/resume individual processes
- **Windows Startup Integration**: Auto-start on boot

## Quick Start

```bash
# Show what would be frozen (dry-run)
smart-freeze.exe

# Run as background daemon (auto-freeze when gaming)
smart-freeze.exe --daemon

# Install to Windows startup
smart-freeze.exe --install-startup

# Keep communication apps running
smart-freeze.exe --daemon --keep-communication

# Manual freeze/resume
smart-freeze.exe --action freeze --pid 1234
smart-freeze.exe --action resume --pid 1234

# Different output formats
smart-freeze.exe --format json
smart-freeze.exe --format csv
```

## Architecture

### Modular Design

```
src/
├── lib.rs                  # Public API + error types
├── main.rs                 # CLI entry point (259 lines)
├── cli.rs                  # Argument parsing
├── process.rs              # Data structures
├── categorization.rs       # Process categorization logic
├── freeze_engine.rs        # Core engine (dependency injection)
├── persistence.rs          # State management (crash recovery)
├── output/                 # Output formatters (table/json/csv)
└── windows/                # Windows-specific implementations
    ├── enumerator.rs       # Process enumeration
    ├── controller.rs       # Freeze/resume control
    └── registry.rs         # Startup management
```

### Process Categories

- **Critical**: System processes (explorer.exe, svchost.exe, dwm.exe, etc.) - Never frozen
- **Gaming**: Game launchers and processes - Protected to maintain performance
- **Communication**: Discord, Teams, Slack - Protected with `--keep-communication`
- **Background**: Google Drive, OneDrive, updaters - Safe to freeze
- **Productivity**: Chrome, Firefox, VS Code, Spotify - Safe to freeze when not foreground

### How It Works

1. **Detection**: Monitors for gaming processes every 60 seconds (configurable)
2. **Freeze**: When game detected, suspends threads of safe-to-freeze processes (>100MB by default)
3. **Resume**: When game exits, resumes all frozen processes
4. **Recovery**: State persisted to disk; auto-resumes on crash/restart

## Testing

```bash
# Run all tests (35 tests, 94% coverage)
cargo test --lib

# Build release
cargo build --release

# Run binary
./target/release/smart-freeze.exe
```

**Test Results**: 35 unit tests, 94% coverage, 0.01s execution time

## Requirements

- Windows 10/11
- Rust 1.70+ (for building from source)
- Administrator privileges (for process suspend/resume)

## Installation

### Via Scoop (Recommended)
```powershell
scoop bucket add napolitain https://github.com/Napolitain/scoop
scoop install baremetal-gaming
```

Scoop will automatically handle updates when new versions are released.

### From Binary Release
1. Download the latest release from [Releases](https://github.com/Napolitain/baremetal-win11/releases)
2. Extract `smart-freeze.exe` from the ZIP
3. Run from command prompt or PowerShell

### From Source
```bash
git clone https://github.com/Napolitain/baremetal-win11.git
cd baremetal-win11
cargo build --release
```

Binary will be in `target/release/smart-freeze.exe`

## Configuration

### Memory Threshold
```bash
# Only freeze processes using >200MB
smart-freeze.exe --threshold 200
```

### Check Interval
```bash
# Check for games every 30 seconds
smart-freeze.exe --daemon --interval 30
```

### Communication Apps
```bash
# Keep Discord, Teams, Slack running
smart-freeze.exe --daemon --keep-communication
```

## Safety Features

- **Crash Recovery**: Frozen processes automatically resumed on startup if daemon crashed
- **Timestamp Validation**: Stale frozen processes (>1 hour) skipped to prevent PID reuse issues
- **Critical Protection**: System processes never touched
- **Foreground Protection**: Active window never frozen
- **Graceful Shutdown**: All processes resumed when daemon exits

## Performance

- **Memory**: ~5-10 MB for daemon
- **CPU**: <0.1% when idle
- **Process Enumeration**: <10ms
- **Freeze/Resume**: <1ms per process
- **Typical Memory Saved**: 2-6 GB during gaming

## Development

### Architecture Principles

- **SOLID**: Single responsibility, dependency injection, interface segregation
- **Trait-Based**: All components mockable for testing
- **Zero-Cost Abstractions**: Traits compile away, no runtime overhead
- **Thread-Safe**: All traits are `Send + Sync`
- **Error Handling**: Structured errors with `thiserror`

### Key Traits

```rust
ProcessEnumerator   // List processes
ProcessController   // Freeze/resume
ProcessCategorizer  // Categorize by importance
StatePersistence    // Save/load state
```

### Adding New Features

- **New Output Format**: Implement `OutputFormatter` trait in `output/`
- **New Categorization**: Extend `ProcessCategorizer` in `categorization.rs`
- **New Storage**: Implement `StatePersistence` trait in `persistence.rs`

## Documentation

- `CHANGELOG.md` - Version history
- `CONTRIBUTING.md` - Development guide
- `DAEMON_MODE.md` - Daemon implementation details
- `QUICK_START.md` - User guide

## License

MIT License - See [LICENSE](LICENSE) file

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md)

## Credits

Built with:
- [windows-sys](https://crates.io/crates/windows-sys) - Win32 API bindings
- [clap](https://crates.io/crates/clap) - CLI parsing
- [serde](https://crates.io/crates/serde) - Serialization
- [thiserror](https://crates.io/crates/thiserror) - Error handling
- [tray-icon](https://crates.io/crates/tray-icon) - System tray
- [winit](https://crates.io/crates/winit) - Event loop
