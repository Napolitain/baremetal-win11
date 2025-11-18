//! CSV output formatting

use crate::cli::Args;
use crate::process::ProcessInfo;
use crate::output::OutputFormatter;

pub struct CsvFormatter;

impl OutputFormatter for CsvFormatter {
    fn format_processes(&self, processes: &[ProcessInfo], args: &Args) {
        println!("PID,Name,MemoryMB,Category,Foreground,FullPath");
        for process in processes {
            println!(
                "{},{},{},{},{},\"{}\"",
                process.pid,
                process.name,
                process.memory_mb,
                process.category.as_str(),
                process.is_foreground,
                process.full_path
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{ProcessCategory, ProcessInfo};

    #[test]
    fn test_csv_output() {
        let formatter = CsvFormatter;
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
            format: crate::cli::OutputFormat::Csv,
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
