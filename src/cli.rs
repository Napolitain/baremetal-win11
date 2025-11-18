//! CLI argument parsing and configuration

use clap::Parser;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "smart-freeze")]
#[command(about = "Smart freeze engine for Windows 11 - intelligently identify heavy but safe-to-freeze applications")]
pub struct Args {
    /// Memory threshold in MB for considering a process "heavy"
    #[arg(short, long, default_value_t = 100)]
    pub threshold: u64,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// Show all processes, not just safe-to-freeze ones
    #[arg(short, long)]
    pub all: bool,

    /// Number of top processes to show by memory usage
    #[arg(short = 'n', long, default_value_t = 10)]
    pub top: usize,

    /// Verbose output (show categorization details)
    #[arg(short, long)]
    pub verbose: bool,

    /// Action to perform on processes
    #[arg(long, value_enum)]
    pub action: Option<Action>,

    /// Process ID to freeze/resume (used with --action)
    #[arg(long)]
    pub pid: Option<u32>,

    /// Run as background daemon with system tray
    #[arg(short, long)]
    pub daemon: bool,

    /// Install to Windows startup (auto-start on boot)
    #[arg(long)]
    pub install_startup: bool,

    /// Uninstall from Windows startup
    #[arg(long)]
    pub uninstall_startup: bool,

    /// Check interval in seconds for daemon mode (default: 60)
    #[arg(long, default_value_t = 60)]
    pub interval: u64,

    /// Keep communication apps running (Discord, Teams, Slack, etc.)
    #[arg(long)]
    pub keep_communication: bool,
}

/// Actions that can be performed on processes
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Action {
    /// Suspend (freeze) a process
    Freeze,
    /// Resume a frozen process
    Resume,
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format
    Json,
    /// CSV format
    Csv,
}
