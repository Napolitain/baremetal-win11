//! Table output formatting

use crate::cli::Args;
use crate::process::{ProcessCategory, ProcessInfo};
use crate::output::OutputFormatter;

pub struct TableFormatter;

impl TableFormatter {
    fn category_to_str(&self, category: ProcessCategory) -> &'static str {
        category.as_str()
    }
}

impl OutputFormatter for TableFormatter {
    fn format_processes(&self, processes: &[ProcessInfo], args: &Args) {
        println!("Smart Freeze Engine - Dry Run Mode");
        println!("===================================\n");

        // Display processes that would be frozen
        if !processes.is_empty() {
            println!("❄️  WOULD FREEZE ({} processes, >{} MB):", processes.len(), args.threshold);
            println!("{}", "=".repeat(70));
            println!("{:<8} {:<40} {:>12} {:<10}", "PID", "Name", "Memory (MB)", "Category");
            println!("{}", "-".repeat(70));

            for process in processes {
                let category_str = self.category_to_str(process.category);
                println!(
                    "{:<8} {:<40} {:>12} {:<10}",
                    process.pid, process.name, process.memory_mb, category_str
                );
            }

            println!(
                "\n   Total memory to free: {} MB",
                processes.iter().map(|p| p.memory_mb).sum::<u64>()
            );
        } else {
            println!("❄️  WOULD FREEZE: None (no processes match criteria)");
        }

        println!("\n💡 This is a DRY RUN. To actually freeze processes, use:");
        println!("   --action freeze --pid <PID>  (manual)");
        println!("   --daemon                     (automatic when gaming)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessCategory;

    #[test]
    fn test_category_to_str() {
        let formatter = TableFormatter;
        assert_eq!(formatter.category_to_str(ProcessCategory::Critical), "Critical");
        assert_eq!(formatter.category_to_str(ProcessCategory::Gaming), "Gaming");
    }

    #[test]
    fn test_empty_output() {
        let formatter = TableFormatter;
        let args = Args {
            threshold: 100,
            format: crate::cli::OutputFormat::Table,
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
        formatter.format_processes(&[], &args);
    }
}
