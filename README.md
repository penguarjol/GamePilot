# GamePilot

**Gaming Intelligence + Auto Clipper for Windows**

GamePilot optimizes your PC game performance and auto-clips your best moments. It detects highlights across 14+ games, scores them with local AI, lets you review and edit, then posts to TikTok, YouTube Shorts, Instagram Reels, and 24 more platforms automatically.

## Install

1. Go to [Releases](../../releases/latest)
2. Download `GamePilot_3.0.0_x64-setup.exe`
3. Run the installer

Everything is bundled: the app, the clip processing engine, and FFmpeg. No Python, no command-line setup, no extra downloads.

Windows SmartScreen may warn on first run (the app is unsigned). Click "More info" then "Run anyway".

### Optional: AI Scoring Rationale

For detailed AI explanations of why each clip scored the way it did, install [Ollama](https://ollama.com) and run:

```
ollama pull qwen3:8b
```

Without Ollama, clip detection and editing still work. Scoring rationale is the only feature that requires it.

### ML Models

Whisper (speech transcription) and YOLO (game event detection) models download automatically on first use. The first clip processing run takes a few extra minutes while models are fetched.

## Features

- **Auto Game Detection** — Detects 14+ games from running processes. Full Steam library scanning.
- **Auto Clipper** — Six detection signals (audio, scene, speech, YOLO, OCR, VLM) find your best moments after each session.
- **Local AI Scoring** — Decima scores clips in 20ms on CPU. Qwen3 provides written rationale. No cloud, no cost.
- **Mood Profiles** — Score clips by intent: Skill, Funny, Montage, or Viral. Same clip, different ranking.
- **Three Editor Modes** — Quick Post (5 seconds), Recipe Editor (30 seconds), Full Timeline (2+ minutes).
- **Trending Content** — Browse TikTok trending sounds, GIPHY stickers, and memes in the editor.
- **Auto-Effects** — Zoom, slow-mo, camera shake, voice mods, captions, and beat-synced cuts applied at key moments.
- **27-Platform Distribution** — Schedule and auto-post via Postiz to TikTok, YouTube Shorts, Reels, X, Discord, Reddit, and more.
- **5 UI Themes** — Phantom Dark, Neon Edge, Carbon Pro, Arctic Clean, Warm Command.
- **Hardware Diagnostics** — CPU, RAM, GPU profiling. Background process detection.
- **Smart Recommendations** — JVM tuning, performance mods, config optimization with backup and rollback.
- **Real-Time Overlay** — Performance metrics and warnings during gameplay.
- **Session Tracking** — Automatic session detection, telemetry, and post-session reports.
- **Mod Store** — Modrinth integration for Minecraft mod management.

## System Requirements

- Windows 10 or later
- 4 GB RAM minimum (8 GB recommended for Clipper)
- NVIDIA GPU recommended for faster detection (CPU fallback works)

## Tech Stack

- **Desktop Runtime:** [Tauri 2](https://tauri.app/) (Rust backend + WebView2)
- **Backend:** Rust
- **Frontend:** React 19 + TypeScript + Tailwind CSS + shadcn/ui
- **Clipper Sidecar:** Python (FastAPI + librosa + PySceneDetect + faster-whisper + YOLO + EasyOCR)
- **Video Processing:** FFmpeg
- **Scoring:** Decima (122M ONNX) + Qwen3-8B (Ollama)
- **Distribution:** Postiz API
- **Database:** SQLite

## Development

### Prerequisites

- [Node.js 22+](https://nodejs.org/) + [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/)
- [Python 3.11+](https://www.python.org/)
- [FFmpeg](https://ffmpeg.org/)
- Platform-specific [Tauri prerequisites](https://tauri.app/start/prerequisites/)

### Setup

```bash
# Install all dependencies (Node + Python sidecar)
./scripts/setup-clipper.sh        # macOS / Linux
.\scripts\setup-clipper.ps1       # Windows PowerShell
```

### Run

```bash
pnpm tauri dev
```

### Build release (with bundled sidecar)

```bash
python sidecar/build.py           # Builds sidecar into src-tauri/binaries/
pnpm tauri build                  # Builds installer with everything bundled
```

### Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Architecture

GamePilot is a game-agnostic platform. The core handles hardware detection, process monitoring, optimization, and safety/rollback. Game modules (Minecraft, League, etc.) plug in with game-specific features.

The **Clipper** module adds post-session clip detection, scoring, editing, and distribution. Detection runs in a Python sidecar (bundled as a native binary via PyInstaller) managed by the Tauri app. Scoring uses Decima for fast first-pass and Qwen3 via Ollama for rationale. The editor supports Quick Post, Recipe, and Full Timeline modes. Distribution goes through Postiz.

## License

This project is not yet licensed. All rights reserved.
