//! SmartFreeze - Main entry point

use clap::Parser;
use smart_freeze::cli::{Action, Args};
use smart_freeze::categorization::DefaultCategorizer;
use smart_freeze::freeze_engine::{FreezeConfig, FreezeEngine};
use smart_freeze::persistence::FileStatePersistence;

#[cfg(windows)]
use smart_freeze::windows::{WindowsProcessController, WindowsProcessEnumerator, WindowsRegistry};

fn main() {
    let args = Args::parse();

    #[cfg(windows)]
    {
        // Handle startup installation/uninstallation
        if args.install_startup {
            handle_install_startup(&args);
            return;
        }

        if args.uninstall_startup {
            handle_uninstall_startup();
            return;
        }

        // Handle daemon mode
        if args.daemon {
            println!("Starting SmartFreeze in daemon mode...");
            println!("Check interval: {} seconds", args.interval);
            println!("Memory threshold: {} MB", args.threshold);
            println!("Keep communication apps: {}", if args.keep_communication { "Yes" } else { "No" });
            println!("System tray icon should appear in taskbar");
            
            // TODO: Implement daemon mode
            eprintln!("Daemon mode not yet implemented in refactored version");
            std::process::exit(1);
        }

        // Handle manual freeze/resume actions
        if let Some(action) = args.action {
            if let Some(pid) = args.pid {
                handle_action(action, pid);
                return;
            } else {
                eprintln!("Error: --pid is required when using --action");
                std::process::exit(1);
            }
        }

        // Default: Show process information
        run_output_mode(&args);
    }

    #[cfg(not(windows))]
    {
        eprintln!("SmartFreeze is only supported on Windows");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn handle_install_startup(args: &Args) {
    let registry = WindowsRegistry::new();
    let exe_path = std::env::current_exe()
        .expect("Failed to get executable path")
        .to_str()
        .expect("Invalid executable path")
        .to_string();

    match registry.install_startup(&exe_path) {
        Ok(()) => {
            println!("✓ SmartFreeze installed to Windows startup");
            println!("  It will auto-start in daemon mode on next boot");
        }
        Err(e) => {
            eprintln!("✗ Failed to install to startup: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
fn handle_uninstall_startup() {
    let registry = WindowsRegistry::new();
    
    match registry.uninstall_startup() {
        Ok(()) => {
            println!("✓ SmartFreeze removed from Windows startup");
        }
        Err(e) => {
            eprintln!("✗ Failed to uninstall from startup: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
fn handle_action(action: Action, pid: u32) {
    use smart_freeze::freeze_engine::ProcessController;
    
    let controller = WindowsProcessController::new();
    
    match action {
        Action::Freeze => {
            match controller.freeze(pid) {
                Ok(count) => {
                    println!("✓ Froze process {} ({} threads suspended)", pid, count);
                }
                Err(e) => {
                    eprintln!("✗ Failed to freeze process {}: {}", pid, e);
                    std::process::exit(1);
                }
            }
        }
        Action::Resume => {
            match controller.resume(pid) {
                Ok(count) => {
                    println!("✓ Resumed process {} ({} threads resumed)", pid, count);
                }
                Err(e) => {
                    eprintln!("✗ Failed to resume process {}: {}", pid, e);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(windows)]
fn run_output_mode(args: &Args) {
    // Create engine with Windows implementations
    let enumerator = WindowsProcessEnumerator::new();
    let controller = WindowsProcessController::new();
    let categorizer = DefaultCategorizer::new();
    
    let config = FreezeConfig {
        min_memory_mb: args.threshold,
        keep_communication: args.keep_communication,
    };

    let mut engine = FreezeEngine::new(enumerator, controller, categorizer, config);

    // Get processes
    match engine.find_safe_to_freeze() {
        Ok(safe_processes) => {
            // Use output formatter
            use smart_freeze::output::{TableFormatter, JsonFormatter, CsvFormatter, OutputFormatter};
            
            match args.format {
                smart_freeze::cli::OutputFormat::Table => {
                    let formatter = TableFormatter;
                    
                    // Enhanced table output with protected processes
                    println!("Smart Freeze Engine - Dry Run Mode");
                    println!("===================================\n");

                    if let Some(fg_pid) = engine.get_foreground_pid() {
                        println!("✓ Foreground Process ID: {}\n", fg_pid);
                    }

                    println!("🎯 DRY RUN - Showing what would happen in daemon mode:\n");

                    if !safe_processes.is_empty() {
                        println!("❄️  WOULD FREEZE ({} processes, >{} MB):", safe_processes.len(), args.threshold);
                        println!("{}", "=".repeat(70));
                        println!("{:<8} {:<40} {:>12} {:<10}", "PID", "Name", "Memory (MB)", "Category");
                        println!("{}", "-".repeat(70));

                        for process in &safe_processes {
                            println!(
                                "{:<8} {:<40} {:>12} {:<10}",
                                process.pid, process.name, process.memory_mb, process.category.as_str()
                            );
                        }

                        println!(
                            "\n   Total memory to free: {} MB",
                            safe_processes.iter().map(|p| p.memory_mb).sum::<u64>()
                        );
                    } else {
                        println!("❄️  WOULD FREEZE: None (no processes match criteria)");
                    }

                    // Show protected processes
                    if let Ok(all_processes) = engine.enumerate_processes() {
                        use smart_freeze::process::ProcessCategory;
                        
                        let protected: Vec<_> = all_processes
                            .iter()
                            .filter(|p| {
                                p.is_foreground 
                                    || p.category == ProcessCategory::Critical 
                                    || p.category == ProcessCategory::Gaming
                            })
                            .collect();

                        if !protected.is_empty() {
                            println!("\n\n🛡️  PROTECTED (will NOT freeze):");
                            println!("{}", "=".repeat(70));
                            println!("{:<8} {:<40} {:>12} {:<10}", "PID", "Name", "Memory (MB)", "Reason");
                            println!("{}", "-".repeat(70));

                            for process in protected.iter().take(20) {
                                let reason = if process.is_foreground {
                                    "Foreground"
                                } else if process.category == ProcessCategory::Critical {
                                    "Critical"
                                } else if process.category == ProcessCategory::Gaming {
                                    "Gaming"
                                } else {
                                    "Unknown"
                                };

                                println!(
                                    "{:<8} {:<40} {:>12} {:<10}",
                                    process.pid, process.name, process.memory_mb, reason
                                );
                            }

                            if protected.len() > 20 {
                                println!("   ... and {} more protected processes", protected.len() - 20);
                            }

                            println!(
                                "\n   Total protected memory: {} MB",
                                protected.iter().map(|p| p.memory_mb).sum::<u64>()
                            );
                        }
                    }

                    println!("\n\n📊 SUMMARY:");
                    println!("{}", "=".repeat(70));
                    if let Ok(all) = engine.enumerate_processes() {
                        println!("   Total processes running: {}", all.len());
                    }
                    println!("   Would freeze: {} processes", safe_processes.len());
                    println!("   Memory threshold: {} MB", args.threshold);
                    println!("\n💡 This is a DRY RUN. To actually freeze processes, use:");
                    println!("   --action freeze --pid <PID>  (manual)");
                    println!("   --daemon                     (automatic when gaming)");
                }
                smart_freeze::cli::OutputFormat::Json => {
                    let formatter = JsonFormatter;
                    formatter.format_processes(&safe_processes, args);
                }
                smart_freeze::cli::OutputFormat::Csv => {
                    let formatter = CsvFormatter;
                    formatter.format_processes(&safe_processes, args);
                }
            }
        }
        Err(e) => {
            eprintln!("Error enumerating processes: {}", e);
            std::process::exit(1);
        }
    }
}
