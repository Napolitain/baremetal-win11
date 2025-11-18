# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.3.0] - 2025-01-18

### Added
- **Daemon Mode**: Background service that automatically freezes processes when gaming
  - System tray integration with menu controls
  - Automatic game detection via continuous monitoring (60 second default interval)
  - Auto-freeze background processes when game launches
  - Auto-resume processes when game exits
  - Enable/disable toggle via system tray
  - Configurable check interval with `--interval` flag
- **Windows Startup Integration**: Registry-based autostart
  - `--install-startup` flag to add to Windows startup
  - `--uninstall-startup` flag to remove from startup
  - Automatically runs in daemon mode on boot
- **Enhanced Critical Process Protection**: Added more system-critical processes
  - TextInputHost.exe (keyboard input)
  - SearchHost.exe (Windows Search)
  - StartMenuExperienceHost.exe (Start Menu)
- **Daemon Mode Documentation**: Comprehensive guide in `DAEMON_MODE.md`

### Changed
- CLI now supports daemon mode alongside existing manual mode
- Process categorization improved for better game detection

### Dependencies Added
- `tray-icon` v0.14 - System tray functionality
- `winit` v0.29 - Event loop for tray UI
- Windows Registry API features

## [0.2.1] - 2025-01-17

### Added
- **CI/CD Pipeline**: Complete GitHub Actions automation
  - Build and test workflow for all branches
  - Automated release workflow with artifact creation
  - Weekly security audit workflow
  - Build artifact uploads (30-day retention)
  - Automated GitHub releases with ZIP archives and checksums
- **Rust Formatting Configuration**: Added `rustfmt.toml` for consistent code style
- **CI/CD Documentation**: Comprehensive workflow documentation in `.github/workflows/README.md`
- **Build Status Badges**: Added badges to README for build and security status

## [0.2.0] - 2025-11-17

### Added
- **CLI Arguments Support**: Added comprehensive command-line argument parsing with `clap`
  - `--threshold`: Configurable memory threshold (default: 100 MB)
  - `--format`: Multiple output formats (table, json, csv)
  - `--all`: Show all processes
  - `--top`: Configure number of top processes to display
  - `--verbose`: Show categorization details
  - `--action`: Freeze or resume processes
  - `--pid`: Target process ID for actions
- **Process Freezing/Resuming**: Implemented actual freeze and resume functionality
  - Freeze processes by suspending all threads
  - Resume frozen processes
  - Thread-level control with proper error handling
- **Multiple Output Formats**: 
  - JSON output for programmatic use
  - CSV output for data analysis
  - Table output (default) for human readability
- **Verbose Mode**: Optional display of process categorization details

### Changed
- **Improved Code Quality**: Fixed all Clippy warnings
  - Simplified `map_or` to direct comparison
  - Changed `last()` to `next_back()` for performance
- **Better Error Handling**: More descriptive error messages
- **Enhanced Documentation**: Updated README with usage examples and CLI options

### Fixed
- Iterator performance issues
- Unnecessary complexity in foreground detection

## [0.1.0] - Initial Release

### Added
- Process enumeration with Win32 APIs
- CPU and memory detection
- Foreground window detection
- Intelligent process categorization
  - Pattern matching on executable names
  - Parent process detection
  - Path analysis for gaming directories
- Smart filtering of safe-to-freeze processes
- Protection of critical system and gaming processes
- Support for Windows 10/11
