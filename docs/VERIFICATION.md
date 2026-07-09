# GamePilot v0.1.2 Verification

## Verification Commands

```bash
# TypeScript
pnpm lint          # tsc --noEmit, zero errors

# Frontend build
pnpm build         # Vite production build, 53 modules, ~272KB JS + 20KB CSS

# Rust check
cargo check --manifest-path src-tauri/Cargo.toml

# Rust tests (11 tests)
cargo test --manifest-path src-tauri/Cargo.toml

# Dev run (macOS)
pnpm tauri dev     # App launches, no panics
```

## Test Coverage

| Test | What it verifies |
|---|---|
| test_parse_prism_instance | mmc-pack.json parsing: MC 1.21.1, NeoForge 21.1.77, JVM config |
| test_parse_prism_mods | Mod detection: ModernFix, FerriteCore, Sodium in jar filenames |
| test_parse_curseforge_instance | minecraftinstance.json: MC 1.20.1, Forge 47.2.0 |
| test_parse_modrinth_instance | profile.json: MC 1.21.1, Fabric 0.16.0 |
| test_parse_empty_instance | Empty folder: no version, no mods, no crash |
| test_manual_folder_selection | Manual folder: 5 mods detected, config path found |
| test_manual_folder_mod_analysis | Sodium/Lithium detected, ModernFix/FerriteCore recommended as missing |
| test_recommendations_generation | 16GB RAM + 250 mods → at least 3 JVM recommendations |
| test_config_analysis | options.txt + server.properties parsing, config recommendations for heavy packs |
| test_modpack_health_scoring | Health score 0-100, non-empty labels for all risk categories |
| test_database_operations | SQLite insert/query round-trip |

## Correctness Fixes in v0.1.2

### Recommendation status values
- Backend accepts: new, accepted, applied, ignored_once, ignored_always, deferred, rolled_back, failed
- Frontend now sends these exact values
- Error handling surfaces failed status updates to the user

### Session lifecycle
- `launch_instance` creates a DB session and returns its ID to the frontend
- `end_session` computes `duration_secs` from `started_at` to current time
- Telemetry samples (CPU/RAM) are accumulated during polling and persisted via `store_session_telemetry` before session end
- Session reports include duration and performance data when available

### Recommendation loading
- Dashboard and Recommendations views use `get_recommendations_for_path` which rescans the instance from disk
- Previously passed SavedInstance JSON which would deserialize incorrectly as a MinecraftInstance

### Apply button honesty
- JVM/config recommendations show "Mark Reviewed" since they do not yet write to launcher config files
- "Open Link" remains for download-link recommendations
- Rollback button only appears when a real file backup exists

## Known Limitations

- **JVM settings are not written to launcher config files.** "Mark Reviewed" acknowledges the recommendation but does not change instance.cfg or launcher settings. This requires launcher-specific config writing which varies by launcher format.
- **FPS telemetry is not implemented.** Session reports show CPU/RAM but not FPS/frame-time. This requires PresentMon/ETW integration or Minecraft log parsing.
- **GPU VRAM/driver detection** uses WMIC on Windows. Returns stub values on macOS.
- **Unsigned installer.** Windows SmartScreen will warn on first launch.
- **Process kill** is recommendation-only. The app never closes processes.
- **Game exit detection** watches for any "java" process. If multiple Java processes run, it may not detect the correct one stopping.
- **Mod metadata** covers ~20 known performance mods. Unknown mods are reported as unclassified.

## Windows CI Verification
- CI runs on windows-latest: lint, build, cargo check, cargo test
- Release workflow builds NSIS and MSI installers on windows-latest
- Installer artifacts are published to GitHub Releases

## Manual Verification Steps

1. Download installer from GitHub Releases
2. Install (SmartScreen: click "More info" > "Run anyway")
3. Open GamePilot — Dashboard loads
4. Navigate to Minecraft > click "Add Instance"
5. Select a folder containing a Minecraft instance or use test fixtures
6. Verify: instance name, path, version, loader, mod count displayed
7. Click "Analyze" — mod analysis, config recommendations, health score appear
8. Review recommendations — at least 3 should appear (RAM, GC flags, missing perf mods)
9. Click "Mark Reviewed" on a recommendation — status changes to "accepted"
10. Click "Launch" — delegates to launcher or opens folder
11. Navigate to Sessions — session record with timestamp visible
12. Navigate to Diagnostics — hardware info, process list, disk info displayed
13. Navigate to Settings — theme toggle, ignore rules, data deletion available
