# Daemon Mode Implementation Status

## Current Status: ⚠️ **Not Yet Migrated**

The daemon mode exists in the old codebase but was **not migrated** during the refactoring. It needs to be ported to use the new modular architecture.

---

## What Daemon Mode Does

### Overview
Background service that runs in system tray and automatically:
1. **Detects** when you launch a game (every 60 seconds)
2. **Freezes** heavy background processes during gaming
3. **Resumes** all frozen processes when you exit the game
4. **Persists** state to disk for crash recovery

### Components Needed

#### 1. **Monitoring Loop**
```rust
// Runs in background thread
loop {
    sleep(interval);
    
    if game_detected() && enabled {
        freeze_background_processes();
    } else if !game_detected() && was_frozen {
        resume_all_frozen_processes();
    }
}
```

#### 2. **System Tray UI**
- Icon in Windows system tray
- Right-click menu:
  - ✅ Enable/Disable auto-freeze
  - ✅ Run on Windows startup
  - ✅ Quit (resumes all and exits)

#### 3. **State Management**
- Track frozen PIDs in memory
- Persist to disk for crash recovery
- Load on startup to resume orphaned processes

---

## Where The Code Is

### Old Implementation (1417-line main.rs)
The daemon was at the bottom of the old main.rs:

```rust
#[cfg(windows)]
mod daemon {
    use tray_icon::{Icon, TrayIconBuilder};
    use winit::event_loop::{ControlFlow, EventLoop};
    
    pub struct DaemonState {
        pub frozen_pids: HashSet<u32>,
        pub game_detected: bool,
        pub enabled: bool,
        pub state_file: PathBuf,
    }
    
    pub fn run_daemon(interval_secs: u64, threshold_mb: u64, keep_communication: bool) {
        // Create state
        let state = Arc::new(Mutex::new(DaemonState::new()));
        
        // Start monitoring thread
        thread::spawn(move || {
            monitor_and_freeze_loop(state, interval_secs, threshold_mb, keep_communication);
        });
        
        // Run system tray UI on main thread
        run_system_tray(state).unwrap();
    }
    
    fn monitor_and_freeze_loop(...) {
        let mut engine = SmartFreezeEngine::new();
        loop {
            sleep(Duration::from_secs(interval_secs));
            
            let game_running = !engine.find_gaming_processes().is_empty();
            
            if game_running && !state.game_detected {
                // Freeze heavy processes
                let safe = engine.find_heavy_processes(threshold_mb);
                for process in safe {
                    freeze_process(process.pid);
                    state.frozen_pids.insert(process.pid);
                }
                state.save(); // Persist to disk
            } else if !game_running && state.game_detected {
                // Resume all
                for pid in state.frozen_pids.drain() {
                    resume_process(pid);
                }
                state.save(); // Clear disk state
            }
        }
    }
}
```

---

## How to Port It

### Step 1: Create `src/daemon/mod.rs`

```rust
//! Daemon mode implementation

mod state;
mod tray;
mod service;

pub use service::run_daemon;
use crate::freeze_engine::FreezeEngine;
use crate::persistence::{FileStatePersistence, StatePersistence};
```

### Step 2: Create `src/daemon/state.rs`

```rust
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct DaemonState {
    pub frozen_pids: HashSet<u32>,
    pub game_detected: bool,
    pub enabled: bool,
}

impl DaemonState {
    pub fn new() -> Self {
        Self {
            frozen_pids: HashSet::new(),
            game_detected: false,
            enabled: true,
        }
    }
}
```

### Step 3: Create `src/daemon/service.rs`

```rust
use super::state::DaemonState;
use super::tray::run_system_tray;
use crate::freeze_engine::{FreezeConfig, FreezeEngine, ProcessController};
use crate::persistence::{FileStatePersistence, PersistentState, StatePersistence};
use crate::windows::{WindowsProcessController, WindowsProcessEnumerator};
use crate::categorization::DefaultCategorizer;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn run_daemon(interval_secs: u64, threshold_mb: u64, keep_communication: bool) {
    // Create persistent state manager
    let persistence = FileStatePersistence::with_default_path();
    
    // Try to recover from previous crash
    if let Ok(Some(old_state)) = persistence.load() {
        println!("[SmartFreeze] Recovering from previous crash...");
        let controller = WindowsProcessController::new();
        for frozen in old_state.get_valid_processes() {
            if let Ok(_) = controller.resume(frozen.pid) {
                println!("[SmartFreeze] Resumed orphaned PID {}", frozen.pid);
            }
        }
        let _ = persistence.delete();
    }
    
    // Create daemon state
    let state = Arc::new(Mutex::new(DaemonState::new()));
    let state_clone = state.clone();
    
    // Start monitoring thread
    thread::spawn(move || {
        monitor_loop(state_clone, interval_secs, threshold_mb, keep_communication);
    });
    
    // Run system tray on main thread
    run_system_tray(state, &persistence).expect("Failed to start system tray");
}

fn monitor_loop(
    state: Arc<Mutex<DaemonState>>,
    interval_secs: u64,
    threshold_mb: u64,
    keep_communication: bool,
) {
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
        
        let mut state = state.lock().unwrap();
        
        if !state.enabled {
            continue;
        }
        
        // Check for gaming processes
        let gaming_running = engine.find_gaming_processes()
            .map(|procs| !procs.is_empty())
            .unwrap_or(false);
        
        if gaming_running && !state.game_detected {
            // Game started - freeze processes
            println!("[SmartFreeze] Game detected! Freezing background processes...");
            state.game_detected = true;
            
            if let Ok(safe) = engine.find_safe_to_freeze() {
                let mut persistent_state = PersistentState::new();
                
                for process in safe {
                    if let Ok(_) = engine.freeze_process(process.pid) {
                        state.frozen_pids.insert(process.pid);
                        persistent_state.add(process.pid, process.name.clone());
                        println!("[SmartFreeze] Froze {} (PID {})", process.name, process.pid);
                    }
                }
                
                // Save to disk for crash recovery
                let _ = persistence.save(&persistent_state);
            }
        } else if !gaming_running && state.game_detected {
            // Game exited - resume all
            println!("[SmartFreeze] No game detected. Resuming processes...");
            state.game_detected = false;
            
            for pid in state.frozen_pids.drain() {
                if let Ok(_) = engine.resume_process(pid) {
                    println!("[SmartFreeze] Resumed PID {}", pid);
                }
            }
            
            // Clear disk state
            let _ = persistence.save(&PersistentState::new());
        }
    }
}
```

### Step 4: Create `src/daemon/tray.rs`

```rust
use super::state::DaemonState;
use crate::persistence::StatePersistence;
use crate::windows::WindowsRegistry;
use std::sync::{Arc, Mutex};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run_system_tray<P: StatePersistence>(
    state: Arc<Mutex<DaemonState>>,
    persistence: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    
    // Create menu items
    let tray_menu = Menu::new();
    let enable_item = MenuItem::new("Enable Auto-Freeze", true, None);
    let startup_item = MenuItem::new("Run on Windows Startup", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    
    tray_menu.append(&enable_item)?;
    tray_menu.append(&startup_item)?;
    tray_menu.append(&quit_item)?;
    
    // Create tray icon
    let icon = Icon::from_rgba(vec![0; 32 * 32 * 4], 32, 32)?;
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_icon(icon)
        .with_tooltip("SmartFreeze")
        .build()?;
    
    // Event loop
    event_loop.run(move |_event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);
        
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == enable_item.id() {
                let mut state = state.lock().unwrap();
                state.enabled = !state.enabled;
                println!("[SmartFreeze] Auto-freeze: {}", if state.enabled { "Enabled" } else { "Disabled" });
            } else if event.id == startup_item.id() {
                // Toggle startup
                let registry = WindowsRegistry::new();
                if registry.is_installed() {
                    let _ = registry.uninstall_startup();
                    println!("[SmartFreeze] Removed from startup");
                } else {
                    let exe_path = std::env::current_exe().unwrap();
                    let _ = registry.install_startup(exe_path.to_str().unwrap());
                    println!("[SmartFreeze] Added to startup");
                }
            } else if event.id == quit_item.id() {
                // Resume all and quit
                println!("[SmartFreeze] Shutting down...");
                let state = state.lock().unwrap();
                // Resume logic here
                let _ = persistence.save(&crate::persistence::PersistentState::new());
                elwt.exit();
            }
        }
    })?;
    
    Ok(())
}
```

### Step 5: Update `main.rs`

```rust
// Handle daemon mode
if args.daemon {
    println!("Starting SmartFreeze in daemon mode...");
    println!("Check interval: {} seconds", args.interval);
    println!("Memory threshold: {} MB", args.threshold);
    println!("Keep communication apps: {}", if args.keep_communication { "Yes" } else { "No" });
    println!("System tray icon should appear in taskbar");
    
    daemon::run_daemon(args.interval, args.threshold, args.keep_communication);
    return;
}
```

---

## What's Missing

To complete daemon mode:

1. **Port the daemon module** (3 files: mod.rs, state.rs, service.rs, tray.rs)
2. **Add to lib.rs**: `pub mod daemon;` (behind `#[cfg(windows)]`)
3. **Update main.rs**: Replace TODO with actual call to `daemon::run_daemon()`
4. **Test**: Run daemon, launch game, verify freeze/resume works

---

## Current Workaround

Until daemon mode is ported, users can:

```bash
# Manual mode still works perfectly
smart-freeze.exe                          # See what would be frozen
smart-freeze.exe --action freeze --pid X  # Freeze manually
smart-freeze.exe --action resume --pid X  # Resume manually

# Registry management works
smart-freeze.exe --install-startup        # (but won't do anything until daemon is ported)
```

---

## Estimated Work

- **Time**: 1-2 hours to port
- **Files**: Create 4 new files in `src/daemon/`
- **Tests**: Add 5-10 tests for daemon state/logic
- **Complexity**: Medium (mostly copy-paste from old main.rs)

The hardest part is done (modular architecture). Daemon is just wiring existing components together!
