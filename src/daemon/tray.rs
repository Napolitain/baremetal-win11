//! System tray UI

use super::state::DaemonState;
use crate::windows::WindowsRegistry;
use std::sync::{Arc, Mutex};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run_system_tray(state: Arc<Mutex<DaemonState>>) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    // Create menu items
    let tray_menu = Menu::new();
    let enable_item = MenuItem::new("Enable Auto-Freeze", true, None);
    let startup_item = MenuItem::new("Run on Windows Startup", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    tray_menu.append(&enable_item)?;
    tray_menu.append(&startup_item)?;
    tray_menu.append(&quit_item)?;

    // Create tray icon (simple blue square)
    let icon_rgba = create_icon_data();
    let icon = Icon::from_rgba(icon_rgba, 32, 32)?;

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_icon(icon)
        .with_tooltip("SmartFreeze - Auto Process Freezer")
        .build()?;

    println!("[SmartFreeze] ✓ System tray icon created");

    // Event loop
    let menu_channel = MenuEvent::receiver();
    let registry = WindowsRegistry::new();

    event_loop.run(move |_event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == enable_item.id() {
                // Toggle auto-freeze
                let mut state_guard = state.lock().unwrap();
                state_guard.toggle_enabled();
                let enabled = state_guard.is_enabled();
                drop(state_guard);

                println!(
                    "[SmartFreeze] Auto-freeze: {}",
                    if enabled { "ENABLED" } else { "DISABLED" }
                );

                // Update menu text
                enable_item.set_text(if enabled {
                    "Disable Auto-Freeze"
                } else {
                    "Enable Auto-Freeze"
                });
            } else if event.id == startup_item.id() {
                // Toggle Windows startup
                if registry.is_installed() {
                    match registry.uninstall_startup() {
                        Ok(()) => {
                            println!("[SmartFreeze] ✓ Removed from Windows startup");
                            startup_item.set_text("Run on Windows Startup");
                        }
                        Err(e) => {
                            eprintln!("[SmartFreeze] ✗ Failed to remove from startup: {}", e);
                        }
                    }
                } else {
                    match std::env::current_exe() {
                        Ok(exe_path) => {
                            if let Some(path_str) = exe_path.to_str() {
                                match registry.install_startup(path_str) {
                                    Ok(()) => {
                                        println!("[SmartFreeze] ✓ Added to Windows startup");
                                        startup_item.set_text("Remove from Windows Startup");
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[SmartFreeze] ✗ Failed to add to startup: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[SmartFreeze] ✗ Failed to get exe path: {}", e);
                        }
                    }
                }
            } else if event.id == quit_item.id() {
                // Quit daemon - restart all terminated processes
                println!("[SmartFreeze] Shutting down...");

                // Load from persistence to get exe paths
                let persistence = crate::persistence::FileStatePersistence::with_default_path();
                use crate::persistence::StatePersistence;

                if let Ok(Some(saved_state)) = persistence.load() {
                    let valid = saved_state.get_valid_processes();
                    if !valid.is_empty() {
                        println!(
                            "[SmartFreeze] Restarting {} terminated processes...",
                            valid.len()
                        );
                        let controller = crate::windows::WindowsProcessController::new();

                        for frozen in valid {
                            match controller.restart_process(&frozen.exe_path) {
                                Ok(new_pid) => println!(
                                    "[SmartFreeze]   ✓ Restarted {} (new PID: {})",
                                    frozen.name, new_pid
                                ),
                                Err(e) => eprintln!(
                                    "[SmartFreeze]   ✗ Failed to restart {}: {}",
                                    frozen.name, e
                                ),
                            }
                        }
                    }
                }

                // Clear persistent state
                let _ = persistence.save(&crate::persistence::PersistentState::new());

                println!("[SmartFreeze] Goodbye!");
                elwt.exit();
            }
        }
    })?;

    Ok(())
}

fn create_icon_data() -> Vec<u8> {
    // Create a simple 32x32 blue square icon
    let mut rgba = Vec::with_capacity(32 * 32 * 4);
    for _ in 0..32 {
        for _ in 0..32 {
            rgba.push(64); // R
            rgba.push(128); // G
            rgba.push(255); // B
            rgba.push(255); // A
        }
    }
    rgba
}
