# GamePilot Implementation Plan

**Status:** In Progress
**Target:** Minecraft MVP — Windows installer + GitHub Pages download site
**Stack:** Tauri 2 + Rust + React/TypeScript + SQLite

## Workstreams

### A. App Scaffold, CI, Packaging
- Tauri 2 project with Rust backend + React/TS frontend
- GitHub Actions: lint/test CI, Windows release build on tags, GitHub Pages deploy
- Unsigned Windows installer (NSIS) via Tauri bundler
- Document SmartScreen behavior for unsigned apps

### B. Desktop UI Shell / Design System
- App shell: sidebar nav, dark-mode-first theme
- Views: Dashboard, Minecraft, Diagnostics, Recommendations, Sessions, Settings
- Design tokens: colors, typography, spacing, radii
- Responsive panes, keyboard nav, loading/error/empty states

### C. Local SQLite Schema / Data Layer
- Migrations via sqlx (embedded)
- Tables: devices, hardware_snapshots, games, game_instances, launch_profiles,
  sessions, telemetry_summaries, process_observations, recommendations,
  optimization_actions, rollback_points, user_preferences, ignore_rules
- Typed Rust data access layer with Tauri commands

### D. Hardware / Process Diagnostics
- Platform adapter trait: HardwareCollector, ProcessAnalyzer
- macOS dev stubs + real Windows implementations
- CPU, RAM, GPU (model/VRAM), disk, OS version detection
- Process list with CPU/RAM usage, known resource-hog signatures
- Tauri commands to expose data to frontend

### E. Minecraft Discovery / Manual Instance Support
- Discover launchers: Prism, CurseForge, Modrinth, ATLauncher, MultiMC, Official
- Parse instance metadata: version, loader (Forge/NeoForge/Fabric/Quilt/Vanilla),
  mods folder, configs, resource packs, shader packs
- Manual folder selection via native file dialog
- Fixture-based tests for each launcher format

### F. Java/JVM Recommendation Rules
- Detect installed Java runtimes
- RAM-based Xmx recommendation table
- GC/JVM flag recommendations (curated rules)
- Performance mod detection (ModernFix, FerriteCore, Sodium, etc.)
- Missing performance mod recommendations
- Config-based recommendations (simulation distance, view distance)

### G. Launch / Session / Report Flow
- Launch profiles: per-instance JVM args, Java path, pre-launch actions
- Preview/apply/rollback for JVM settings with file backup
- Launch delegation: open instance in detected launcher or direct Java launch
- Session lifecycle: start time, end time, hardware snapshot, process snapshot
- Post-session report generation

### H. GitHub Pages Product/Download Site
- Static site with product info
- Placeholder screenshots
- Download link to latest GitHub Release
- Build/deploy via GitHub Actions

## Build Order

1. Scaffold (A) — foundation for everything
2. CI (A) — validate builds early
3. Database (C) — data layer needed by features
4. UI shell (B) — navigation and layout
5. Hardware/Process (D) — first real feature
6. Minecraft discovery (E) — core value prop
7. JVM rules (F) — recommendations
8. Launch/Session (G) — complete the loop
9. GitHub Pages (H) — product site
10. Polish, testing, verification

## Platform Adapter Strategy

Each OS-specific module has:
- `mod.rs` — shared trait/interface
- `windows.rs` — real Windows implementation
- `macos.rs` — dev stub/fallback for local development
- Selection via `#[cfg(target_os)]` at compile time

CI builds on `windows-latest` to verify real Windows code.
