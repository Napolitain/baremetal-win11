//! SmartFreeze - Intelligent Windows Process Freezer
//!
//! This library provides the core functionality for detecting and freezing
//! background processes to optimize system performance during gaming.

pub mod categorization;
pub mod cli;
pub mod freeze_engine;
pub mod output;
pub mod persistence;
pub mod process;

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub mod daemon;

pub use categorization::ProcessCategorizer;
pub use freeze_engine::FreezeEngine;
pub use process::{ProcessCategory, ProcessInfo};

/// Result type for SmartFreeze operations
pub type Result<T> = std::result::Result<T, SmartFreezeError>;

/// Error types for SmartFreeze operations
#[derive(Debug, thiserror::Error)]
pub enum SmartFreezeError {
    #[error("Failed to enumerate processes: {0}")]
    ProcessEnumeration(String),

    #[error("Failed to freeze process {pid}: {reason}")]
    FreezeFailed { pid: u32, reason: String },

    #[error("Failed to resume process {pid}: {reason}")]
    ResumeFailed { pid: u32, reason: String },

    #[error("Process not found: {0}")]
    ProcessNotFound(u32),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Registry error: {0}")]
    Registry(String),
}
