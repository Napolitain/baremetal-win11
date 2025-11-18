//! JSON output formatting

use crate::cli::Args;
use crate::process::ProcessInfo;
use crate::output::OutputFormatter;
use serde_json::json;

pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format_processes(&self, processes: &[ProcessInfo], args: &Args) {
        let output = json!({
            "threshold_mb": args.threshold,
            "safe_to_freeze_count": processes.len(),
            "total_memory_mb": processes.iter().map(|p| p.memory_mb).sum::<u64>(),
            "processes": processes,
        });
        
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{ProcessCategory, ProcessInfo};

    #[test]
    fn test_json_output() {
        let formatter = JsonFormatter;
        let processes = vec![
            ProcessInfo::new(
                1234,
                "test.exe".to_string(),
                "C:\\test.exe".to_string(),
                200,
                false,
                ProcessCategory::Productivity,
            ),
        ];
        
        let args = Args {
            threshold: 100,
            format: crate::cli::OutputFormat::Json,
            all: false,
            top: 10,
            verbose: false,
            action: None,
            pid: None,
            daemon: false,
            install_startup: false,
            uninstall_startup: false,
            interval: 60,
            keep_communication: false,
        };
        
        // Should not panic
        formatter.format_processes(&processes, &args);
    }
}
