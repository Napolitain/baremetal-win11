//! Daemon service implementation

use super::state::DaemonState;
use super::tray::run_system_tray;
use crate::categorization::DefaultCategorizer;
use crate::freeze_engine::{FreezeConfig, FreezeEngine, ProcessController};
use crate::persistence::{FileStatePersistence, PersistentState, StatePersistence};
use crate::windows::{WindowsProcessController, WindowsProcessEnumerator};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Run daemon mode
pub fn run_daemon(interval_secs: u64, threshold_mb: u64, keep_communication: bool) {
    // Create persistent state manager
    let persistence = FileStatePersistence::with_default_path();

    // Try to recover from previous crash
    recover_from_crash(&persistence);

    // Create daemon state
    let state = Arc::new(Mutex::new(DaemonState::new()));
    let state_clone = state.clone();

    // Start monitoring thread
    thread::spawn(move || {
        monitor_loop(state_clone, interval_secs, threshold_mb, keep_communication);
    });

    // Run system tray on main thread
    println!("[SmartFreeze] Starting system tray...");
    if let Err(e) = run_system_tray(state) {
        eprintln!("[SmartFreeze] System tray error: {}", e);
    }
}

fn recover_from_crash(persistence: &FileStatePersistence) {
    if let Ok(Some(old_state)) = persistence.load() {
        let valid = old_state.get_valid_processes();
        if !valid.is_empty() {
            println!(
                "[SmartFreeze] Recovering from previous crash ({} frozen processes)...",
                valid.len()
            );
            let controller = WindowsProcessController::new();
            let mut resumed = 0;
            let mut failed = 0;

            for frozen in valid {
                match controller.resume(frozen.pid) {
                    Ok(_) => {
                        println!("[SmartFreeze] ✓ Resumed {} (PID {})", frozen.name, frozen.pid);
                        resumed += 1;
                    }
                    Err(_) => {
                        failed += 1;
                    }
                }
            }

            println!(
                "[SmartFreeze] Recovery complete: {} resumed, {} skipped",
                resumed, failed
            );
        }
        let _ = persistence.delete();
    }
}

fn monitor_loop(
    state: Arc<Mutex<DaemonState>>,
    interval_secs: u64,
    threshold_mb: u64,
    keep_communication: bool,
) {
    println!("[SmartFreeze] Monitoring thread started");
    println!("[SmartFreeze] Check interval: {}s", interval_secs);
    println!("[SmartFreeze] Memory threshold: {}MB", threshold_mb);
    println!(
        "[SmartFreeze] Communication protection: {}",
        if keep_communication { "ON" } else { "OFF" }
    );

    let persistence = FileStatePersistence::with_default_path();
    let enumerator = WindowsProcessEnumerator::new();
    let controller = WindowsProcessController::new();
    let categorizer = DefaultCategorizer::new();

    let config = FreezeConfig {
        min_memory_mb: threshold_mb,
        keep_communication,
    };

    let mut engine = FreezeEngine::new(enumerator, controller, categorizer, config);

    loop {
        thread::sleep(Duration::from_secs(interval_secs));

        let mut state_guard = state.lock().unwrap();

        if !state_guard.is_enabled() {
            continue;
        }

        // Check for gaming processes
        let gaming_running = engine
            .find_gaming_processes()
            .map(|procs| !procs.is_empty())
            .unwrap_or(false);

        if gaming_running && !state_guard.game_detected {
            // Game started - freeze processes
            println!("[SmartFreeze] 🎮 Game detected! Freezing background processes...");
            state_guard.game_detected = true;

            if let Ok(safe) = engine.find_safe_to_freeze() {
                let mut persistent_state = PersistentState::new();
                let mut frozen_count = 0;
                let mut total_memory = 0u64;

                for process in safe {
                    match engine.freeze_process(process.pid) {
                        Ok(_) => {
                            state_guard.add_frozen(process.pid);
                            persistent_state.add(process.pid, process.name.clone());
                            total_memory += process.memory_mb;
                            frozen_count += 1;
                            println!(
                                "[SmartFreeze]   ❄️  Froze {} (PID {}, {} MB)",
                                process.name, process.pid, process.memory_mb
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[SmartFreeze]   ✗ Failed to freeze {} (PID {}): {}",
                                process.name, process.pid, e
                            );
                        }
                    }
                }

                // Save to disk for crash recovery
                if let Err(e) = persistence.save(&persistent_state) {
                    eprintln!("[SmartFreeze] Warning: Failed to save state: {}", e);
                }

                println!(
                    "[SmartFreeze] ✓ Froze {} processes, freed ~{} MB",
                    frozen_count, total_memory
                );
            } else {
                eprintln!("[SmartFreeze] Failed to enumerate safe processes");
            }
        } else if !gaming_running && state_guard.game_detected {
            // Game exited - resume all
            println!("[SmartFreeze] 🎮 Game closed. Resuming processes...");
            state_guard.game_detected = false;

            let pids = state_guard.clear_frozen();
            let mut resumed_count = 0;

            for pid in pids {
                match engine.resume_process(pid) {
                    Ok(_) => {
                        println!("[SmartFreeze]   ✓ Resumed PID {}", pid);
                        resumed_count += 1;
                    }
                    Err(e) => {
                        eprintln!("[SmartFreeze]   ✗ Failed to resume PID {}: {}", pid, e);
                    }
                }
            }

            // Clear disk state
            if let Err(e) = persistence.save(&PersistentState::new()) {
                eprintln!("[SmartFreeze] Warning: Failed to clear state: {}", e);
            }

            println!("[SmartFreeze] ✓ Resumed {} processes", resumed_count);
        }
    }
}
