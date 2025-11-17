# Process Categorization Strategies

This document details the strategies used by the smart freeze engine to categorize processes into importance levels.

## Overview

The engine categorizes processes into 6 categories:
1. **Critical** - System processes (never freeze)
2. **Gaming** - Game-related processes (important to keep)
3. **Communication** - Chat/voice apps (potentially important)
4. **Background Service** - Launchers and utilities (safe to freeze)
5. **Productivity** - Browsers and apps (safe to freeze when not foreground)
6. **Unknown** - Uncategorized processes

## Strategy 1: Pattern Matching on Executable Names

**Primary Method**: Match process executable names against known patterns.

**Advantages**:
- Fast and reliable
- Works for all known applications
- No additional API calls needed

**Limitations**:
- Requires maintaining pattern lists
- Cannot detect custom/unknown applications
- May not work for renamed executables

### Implementation

```rust
fn categorize_process(&self, name: &str) -> ProcessCategory {
    let name_lower = name.to_lowercase();
    
    // Check against pattern lists
    if matches_gaming_patterns(&name_lower) {
        return ProcessCategory::Gaming;
    }
    // ... etc
}
```

## Category Details

### Critical System Processes

**Never freeze these** - Essential for Windows operation.

**Patterns**:
- System, smss.exe, csrss.exe, wininit.exe
- services.exe, lsass.exe, svchost.exe
- winlogon.exe, explorer.exe, dwm.exe

**Rationale**: Freezing these would cause system instability or crashes.

### Gaming Processes (Important to Keep)

**Keep responsive** - Games and related services should not be frozen.

**Categories**:

1. **Game Launchers**:
   - Steam (steam.exe, steamwebhelper.exe, steamservice.exe)
   - Epic Games (epicgameslauncher.exe, epicwebhelper.exe)
   - Origin (origin.exe, originwebhelper.exe)
   - GOG Galaxy (gog.exe, galaxyclient.exe)
   - Battle.net (battle.net.exe, blizzard.exe)

2. **Anti-cheat Systems**:
   - EasyAntiCheat (easyanticheat.exe)
   - BattlEye (battleye.exe)
   - Vanguard (vanguard.exe)

3. **Game Executables**:
   - Common patterns: game.exe, launcher.exe
   - Specific games: csgo.exe, dota2.exe, leagueoflegends.exe, etc.

**Rationale**: 
- Freezing game processes causes lag and poor gaming experience
- Anti-cheat systems may flag frozen processes as tampering
- Game launchers manage game state and updates

**Why Not Freeze**:
- Player experience is paramount
- Games often run time-critical loops
- Multiplayer games need consistent connection

### Communication Apps (Potentially Important)

**Context-dependent** - Important during active use, less so when idle.

**Patterns**:
- Discord (discord.exe, discordcanary.exe, discordptb.exe)
- Slack (slack.exe)
- Microsoft Teams (teams.exe)
- Zoom (zoom.exe)
- Matrix clients (element.exe, riot.exe)
- Telegram, Signal, WhatsApp, etc.

**Rationale**:
- Users want notifications from these apps
- May be in active calls or chats
- Less critical than games but more than browsers

**When Safe to Freeze**:
- Not in active call/chat (hard to detect automatically)
- Using significant memory (>100 MB)
- Not the foreground application

**Current Implementation**: 
Communication apps are allowed to be frozen if they meet memory threshold and aren't foreground. Future enhancement could detect active calls via audio stream detection.

### Background Services (Safe to Freeze)

**Usually safe** - Background launchers and utilities that aren't actively needed.

**Categories**:

1. **Game Launchers (Background Mode)**:
   - Ubisoft Connect (uplay.exe, upc.exe, ubisoftconnect.exe)
   - Epic Online Services (epiconlineservices.exe)

2. **Graphics Utilities**:
   - NVIDIA (nvcontainer.exe, nvidia share.exe, geforce experience.exe)
   - AMD (amdrsserv.exe, radeonsoft.exe)

3. **Developer Tools**:
   - JetBrains Toolbox (jetbrains.toolbox.exe)

4. **Updaters and Helpers**:
   - update.exe, updater.exe, helper.exe
   - crashhandler.exe, crashreporter.exe

5. **Cloud Sync**:
   - OneDrive, Dropbox, Google Drive Sync

**Rationale**:
- These run in background without user interaction
- Not time-critical
- Can be resumed when needed

**Why Safe to Freeze**:
- No user-facing functionality when idle
- Don't affect gaming or active work
- Save significant memory (often 100-500 MB)

### Productivity/Browsers (Safe When Not Foreground)

**Context-dependent** - Essential when in use, safe to freeze otherwise.

**Categories**:

1. **Browsers**:
   - Chrome, Firefox, Edge, Opera, Brave, Vivaldi

2. **Office Applications**:
   - Excel, Word, PowerPoint, Outlook, OneNote

3. **IDEs and Editors**:
   - VSCode, PyCharm, IntelliJ, Rider
   - Sublime Text, Atom, Notepad++

4. **Media Players**:
   - Spotify, VLC, iTunes, MusicBee

5. **Note-taking Apps**:
   - Notion, Obsidian

**Rationale**:
- Heavy memory users (browsers especially)
- Not time-critical when in background
- Can be resumed without data loss

**Why Safe to Freeze**:
- Modern apps handle suspension well
- Browsers save state automatically
- Background tabs don't need CPU

## Future Enhancement Strategies

### Strategy 2: Parent Process Detection (Not Yet Implemented)

**Concept**: Identify processes by their parent process.

**Example**:
```rust
// Games launched by Steam have Steam as parent
if parent_process_name == "steam.exe" {
    return ProcessCategory::Gaming;
}
```

**Advantages**:
- Catches unknown game executables
- More comprehensive than name matching

**Implementation Complexity**: Requires additional Win32 API calls to get parent PID.

### Strategy 3: Path Analysis (Not Yet Implemented)

**Concept**: Categorize based on installation path.

**Example**:
```rust
if path.starts_with("C:\\Program Files\\Steam\\") {
    return ProcessCategory::Gaming;
}
```

**Advantages**:
- Works for custom executables
- Can identify installation type

**Limitations**:
- Games can be installed anywhere
- Requires full path query (additional API call)

### Strategy 4: Resource Usage Patterns (Not Yet Implemented)

**Concept**: Identify process type by behavior.

**Patterns**:
- **Games**: High GPU usage, consistent frame timing
- **Browsers**: Many child processes, varied memory usage
- **Background Services**: Low CPU, steady memory

**Advantages**:
- Works for unknown applications
- More accurate categorization

**Implementation Complexity**: Requires multiple samples over time, GPU API access.

### Strategy 5: Process Tree Analysis (Not Yet Implemented)

**Concept**: Analyze relationships between processes.

**Example**:
```
chrome.exe (parent)
├── chrome.exe (renderer)
├── chrome.exe (renderer)
└── chrome.exe (GPU process)
```

**Advantages**:
- Better understanding of process relationships
- Can freeze entire process trees

**Implementation**: Requires building process tree from parent-child relationships.

## Categorization Decision Flow

```
1. Is process in critical list?
   YES → Critical (never freeze)
   NO → Continue

2. Does name match gaming patterns?
   YES → Gaming (important to keep)
   NO → Continue

3. Does name match communication patterns?
   YES → Communication (potentially important)
   NO → Continue

4. Does name match background service patterns?
   YES → Background Service (safe to freeze)
   NO → Continue

5. Does name match productivity patterns?
   YES → Productivity (safe when not foreground)
   NO → Unknown
```

## Freeze Decision Logic

```rust
fn is_safe_to_freeze(process: &ProcessInfo) -> bool {
    // Never freeze if foreground
    if process.is_foreground {
        return false;
    }
    
    // Never freeze critical or gaming
    if process.category == Critical || process.category == Gaming {
        return false;
    }
    
    // Freeze if using significant memory
    process.memory_mb >= threshold
}
```

## Testing Recommendations

When testing categorization:

1. **Verify Critical Protection**: Ensure system processes are never suggested
2. **Test Gaming Scenario**: Run a game, verify it's not in freeze list
3. **Test Communication**: Check Discord/Slack behavior during calls vs idle
4. **Test Background Services**: Verify Ubisoft Connect, GeForce Experience are listed
5. **Test Browsers**: Ensure Chrome/Firefox can be frozen when not foreground

## Maintenance

To add new applications:

1. Identify the executable name (e.g., `newapp.exe`)
2. Determine the appropriate category
3. Add to the relevant pattern list in `categorize_process()`
4. Test with the actual application

## Contribution Guidelines

When adding patterns:
- Use lowercase for comparisons
- Be specific (avoid overly broad patterns)
- Document why the categorization was chosen
- Test with the actual application if possible
