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
- [Python 3.11+](https://www.python.org/) (for the Clipper sidecar)
- [FFmpeg](https://ffmpeg.org/) (for clip processing)
- Platform-specific Tauri dependencies ([see Tauri docs](https://tauri.app/start/prerequisites/))

### Setup

**Quick start (includes Clipper sidecar):**

```bash
# macOS / Linux
./scripts/setup-clipper.sh

# Windows (PowerShell)
.\scripts\setup-clipper.ps1
```

This handles: Node deps, Python venv, sidecar dependencies, FFmpeg check, and model directory.

**Manual setup (frontend only, no Clipper):**

```bash
pnpm install
```

### Optional: Ollama for AI Scoring Rationale

The Clipper's scoring rationale requires Ollama with Qwen3. Install from [ollama.com](https://ollama.com), then:

```bash
ollama pull qwen3:8b
```

Without Ollama, clip detection and extraction still work. Scoring rationale and mood-aware analysis are skipped.

### Run in development

```bash
pnpm tauri dev
```

The Clipper sidecar (Python) starts automatically with the app. If the sidecar fails to start (missing deps or FFmpeg), the rest of GamePilot still works normally — Clipper features are disabled gracefully.

### Build for production

```bash
pnpm tauri build
```

The installer is output to `src-tauri/target/release/bundle/nsis/`.

### Run tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Verify Clipper sidecar setup

```bash
python3 sidecar/setup.py --check
```

### Project structure

```
src/                    # React/TypeScript frontend
  components/           # UI components (Sidebar, etc.)
  views/                # Page views (Dashboard, Clips, Review, Editor, etc.)
  styles/               # CSS theme and design tokens (includes 5 clipper themes)
  hooks/                # Custom React hooks
src-tauri/              # Rust backend
  src/
    clipper/            # Clip processing module
      sidecar.rs        # Python sidecar lifecycle management
      clips.rs          # Clip CRUD and metadata
      composition.rs    # Editor composition types
      scoring.rs        # Decima + Qwen3 scoring cascade
      watcher.rs        # File system watcher for recordings
      pipeline.rs       # Processing orchestration
      distribution.rs   # Postiz social media integration
      teaching.rs       # Feedback loop and weight calibration
      trending.rs       # Trending content API clients
      recorder.rs       # Recording configuration
    db/                 # SQLite database and schema
    hardware/           # Hardware detection, process analyzer
    minecraft/          # Instance discovery, mod analysis, JVM rules
    launch/             # Launch profiles, instance launching
    sessions/           # Session tracking and reports
    recommendations/    # Backup/rollback system
    platform/           # Platform-specific adapters (Java detection)
    lib.rs              # Tauri commands and app setup
sidecar/                # Python ML sidecar (FastAPI)
  api.py                # Server entrypoint (/health, /detect, /process, /transcribe)
  setup.py              # Dependency installer and model downloader
  detection/            # Audio, scene, whisper, YOLO, OCR, VLM modules
  processing/           # FFmpeg effects, recipes, compilations, thumbnails
  models/               # ML model weights (downloaded on first use)
scripts/                # Setup and automation scripts
tests/
  fixtures/             # Test fixtures for Minecraft instances
docs/                   # PRD, ADR, implementation docs
site/                   # GitHub Pages product site
.github/workflows/      # CI, release, and Pages deployment
```

## Architecture

GamePilot follows a game-agnostic core architecture. The core platform handles hardware detection, process monitoring, recommendation orchestration, optimization execution, and safety/rollback. Minecraft is implemented as the first game module.

The **Clipper** module adds post-session clip detection, scoring, editing, and distribution. Detection runs in a Python sidecar (FastAPI on localhost) managed by the Tauri app. Scoring uses Decima (122M ONNX, CPU) for fast first-pass and Qwen3-8B via Ollama for rationale. The editor supports three modes: Quick Post (5s), Recipe Editor (30s), and Full Timeline (2+ min). Distribution goes through Postiz (self-hosted, 27 platforms).

See [docs/ADR.md](docs/ADR.md) for architecture decision records and [docs/PRD.md](docs/PRD.md) for the full product requirements.

## License

This project is not yet licensed. All rights reserved.
