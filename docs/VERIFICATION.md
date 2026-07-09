# GamePilot MVP Verification

## Verification Steps

### 1. Install the built Windows app from GitHub Release
- Download the `.exe` installer from the GitHub Release page
- Windows SmartScreen will show a warning (app is unsigned)
- Click "More info" then "Run anyway" to proceed
- The installer runs in per-user mode (no admin required)
- GamePilot appears in Start Menu after installation

### 2. Open GamePilot
- Launch from Start Menu or desktop shortcut
- Dashboard should load within 2 seconds
- Left sidebar shows navigation: Dashboard, Minecraft, Diagnostics, Recommendations, Sessions, Settings

### 3. Use manual folder selection to choose a Minecraft instance
- Navigate to the Minecraft view
- Click "Add Instance"
- Select a folder containing a Minecraft instance (with mods/, config/, etc.)
- Alternatively, select one of the test fixture folders

### 4. Detect instance properties
After selecting a folder, verify:
- Instance path is displayed
- Minecraft version is detected (if available via mmc-pack.json, minecraftinstance.json, or profile.json)
- Loader is detected (Forge, NeoForge, Fabric, Quilt, or Vanilla)
- Mods folder contents are listed with count
- Java/JVM configuration is displayed (if available from instance.cfg)

### 5. Run diagnostics
- Navigate to Diagnostics view
- Verify hardware info is displayed: CPU model, cores, threads, RAM total/used/available
- Verify process list shows running processes with CPU/RAM usage
- Resource hogs are highlighted (Chrome, Discord, OBS, etc.)
- Java installations are listed

### 6. Show at least three real recommendations
After analyzing an instance, verify at least three recommendations:
- RAM/JVM recommendation (based on system RAM and mod count)
- GC flag recommendation (optimized G1GC flags)
- Missing performance mod recommendation (e.g., ModernFix, FerriteCore, Entity Culling)

Each recommendation includes:
- Title and description
- Evidence (system RAM, current settings, mod count)
- Confidence level (high/medium/low)
- Risk level
- Expected impact

### 7. Preview at least one safe optimization
- Select a JVM recommendation
- Preview shows the proposed change (e.g., new Xmx value, GC flags)
- The action is clearly labeled with risk level

### 8. Apply and roll back at least one change
- Apply a JVM settings change
- Verify a backup is created in `.gamepilot_backups/`
- Rollback the change
- Verify the original file is restored

### 9. Launch Minecraft or delegate
- Click "Launch" on an instance
- GamePilot attempts to launch via the detected launcher (Prism, CurseForge, etc.)
- If the launcher is not found, GamePilot opens the instance folder
- A session record is created

### 10. Generate a session/report record
- After launching, a session appears in the Sessions view
- The session shows: instance name, start time, launch method, status
- The report includes: recommendation count, process observation count

### 11. Confirm app responsiveness
- The app remains responsive during all operations
- No expensive background work runs during gameplay
- Process scanning completes in under 2 seconds

## Test Fixtures

Test fixtures are available in `tests/fixtures/`:

| Fixture | Description | Expected Detection |
|---|---|---|
| `prism-instance` | Prism/MultiMC format with mmc-pack.json | MC 1.21.1, NeoForge 21.1.77, 10 mods |
| `curseforge-instance` | CurseForge format with minecraftinstance.json | MC 1.20.1, Forge 47.2.0 |
| `modrinth-instance` | Modrinth format with profile.json | MC 1.21.1, Fabric 0.16.0 |
| `manual-folder` | Plain folder with mods/ and config/ | 5 mods, Sodium + Lithium detected |
| `empty-instance` | Empty folder | No version, no mods |

## Automated Test Results

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Tests cover:
- Prism instance parsing (version, loader, JVM config)
- Prism mod detection (ModernFix, FerriteCore, Sodium)
- CurseForge instance parsing
- Modrinth instance parsing
- Empty instance handling
- Manual folder selection
- Manual folder mod analysis
- Recommendation generation (minimum 3 recommendations)
- Database operations

## Known Limitations

- FPS telemetry is deferred (CPU/RAM/process monitoring is implemented)
- GPU model detection uses WMIC on Windows; stub on macOS
- Automatic launcher discovery depends on standard install paths
- Launch delegation relies on launcher CLI availability
- No code signing (SmartScreen warning on Windows)
- Session duration tracking requires manual session end
- Mod metadata is limited to ~20 known performance mods
- Config file analysis is basic (JVM args only for MVP)

## Windows-Specific Notes

- Hardware detection uses `sysinfo` crate (cross-platform)
- GPU detection uses `wmic` command on Windows
- Process monitoring uses `sysinfo` crate
- Launcher discovery checks `%APPDATA%` and `%LOCALAPPDATA%` paths
- File dialog uses Tauri's native dialog plugin (WebView2 on Windows)
