//! Daemon mode - automatic process freezing when gaming

mod service;
mod state;
mod tray;

pub use service::run_daemon;
pub use state::DaemonState;
