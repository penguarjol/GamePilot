# GamePilot

**Intelligent Minecraft Performance Optimization for Windows**

GamePilot is a lightweight Windows desktop app that helps Minecraft players get the best performance from their setup. It discovers your Minecraft instances, analyzes modpacks, detects hardware and resource bottlenecks, recommends safe optimizations, and tracks your gaming sessions.

## Features

- **Instance Discovery** — Automatically detects Minecraft installations from Prism Launcher, CurseForge, Modrinth App, ATLauncher, MultiMC, and the official launcher. Manual folder selection also supported.
- **Hardware Diagnostics** — Detects CPU, RAM, GPU, and OS details. Identifies resource-heavy background processes (Chrome, Discord, OBS, OneDrive, etc.) that compete with your game.
- **Smart Recommendations** — Rules-based JVM tuning (Xmx, GC flags, Java version), missing performance mod detection (ModernFix, FerriteCore, Sodium, etc.), and modpack health analysis.
- **Safe Optimizations** — Every change includes preview, backup, apply, and rollback. No silent modifications.
- **Launch Integration** — Launch instances through your preferred launcher or open the instance folder for manual launch.
- **Session Tracking** — Records session history with hardware snapshots and generates post-session reports.

## Tech Stack

- **Desktop Runtime:** [Tauri 2](https://tauri.app/) (Rust backend + WebView2)
- **Backend:** Rust
- **Frontend:** React 19 + TypeScript
- **Build:** Vite
- **Database:** SQLite (via rusqlite)
- **Packaging:** NSIS installer for Windows

## Install

### From GitHub Releases

1. Go to [Releases](../../releases/latest)
2. Download `GamePilot_0.1.0_x64-setup.exe`
3. Run the installer (Windows SmartScreen may warn — click "More info" then "Run anyway"; the app is unsigned)
4. Launch GamePilot from the Start Menu

### System Requirements

- Windows 10 or later
- 4 GB RAM minimum
- WebView2 Runtime (included with Windows 10 21H2+ and Windows 11)

## Development

### Prerequisites

- [Node.js 22+](https://nodejs.org/)
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/)
- Platform-specific Tauri dependencies ([see Tauri docs](https://tauri.app/start/prerequisites/))

### Setup

```bash
pnpm install
```

### Run in development

```bash
pnpm tauri dev
```

### Build for production

```bash
pnpm tauri build
```

The installer is output to `src-tauri/target/release/bundle/nsis/`.

### Run tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Project structure

```
src/                    # React/TypeScript frontend
  components/           # UI components (Sidebar, etc.)
  views/                # Page views (Dashboard, Minecraft, Diagnostics, etc.)
  styles/               # CSS theme and design tokens
  hooks/                # Custom React hooks
src-tauri/              # Rust backend
  src/
    db/                 # SQLite database and schema
    hardware/           # Hardware detection, process analyzer
    minecraft/          # Instance discovery, mod analysis, JVM rules
    launch/             # Launch profiles, instance launching
    sessions/           # Session tracking and reports
    recommendations/    # Backup/rollback system
    platform/           # Platform-specific adapters (Java detection)
    lib.rs              # Tauri commands and app setup
tests/
  fixtures/             # Test fixtures for Minecraft instances
docs/                   # PRD, ADR, implementation docs
site/                   # GitHub Pages product site
.github/workflows/      # CI, release, and Pages deployment
```

## Architecture

GamePilot follows a game-agnostic core architecture. The core platform handles hardware detection, process monitoring, recommendation orchestration, optimization execution, and safety/rollback. Minecraft is implemented as the first game module.

See [docs/ADR.md](docs/ADR.md) for architecture decision records and [docs/PRD.md](docs/PRD.md) for the full product requirements.

## License

This project is not yet licensed. All rights reserved.
