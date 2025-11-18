//! Daemon mode - automatic process freezing when gaming

mod state;
mod service;
mod tray;

pub use service::run_daemon;
pub use state::DaemonState;
