# GamePilot MVP Status

**Last Updated:** 2026-07-08
**Current Release:** v0.1.2

## Overall: STABLE

| Workstream | Status | Notes |
|---|---|---|
| A. Scaffold + CI | Done | Tauri 2 + Rust + React/TS, 3 GitHub Actions workflows |
| B. UI Shell | Done | Dashboard, Minecraft, Diagnostics, Recommendations, Sessions, Settings |
| C. SQLite/Data | Done | 8 tables, migrations on startup, preferences store |
| D. Hardware/Process | Done | CPU/RAM/GPU/disk detection, 20 resource-hog signatures |
| E. Minecraft Discovery | Done | Prism, CurseForge, Modrinth, ATLauncher, MultiMC, Official, Custom |
| F. JVM/Config Rules | Done | RAM recs, GC flags, Java version, perf mods, options.txt, server.properties |
| G. Launch/Session | Done | Launcher delegation, session lifecycle, duration, telemetry persistence |
| H. GitHub Pages | Done | Product site with accurate version and download link |

## v0.1.2 Fixes (from v0.2.0 WIP)
- Fixed recommendation status values to match backend enum (ignored_once, ignored_always)
- Fixed session ID mismatch between launch and DB
- Fixed duration_secs computation in end_session
- Fixed recommendation loading from saved instances (new get_recommendations_for_path command)
- Renamed misleading "Apply" button to "Mark Reviewed" for JVM recommendations
- Added telemetry persistence (CPU/RAM averages stored on session end)
- Added error handling for failed status updates
- Removed dead rollback UI for non-file-backed recommendations

## Release Artifacts
- **GitHub Release:** https://github.com/penguarjol/GamePilot/releases
- **GitHub Pages:** https://penguarjol.github.io/GamePilot/

## Test Results
- 11 Rust tests passing
- TypeScript compiles cleanly
- Vite build succeeds
- CI passes on ubuntu-latest and windows-latest
