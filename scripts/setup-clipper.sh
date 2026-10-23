#!/usr/bin/env bash
# GamePilot Clipper — One-command setup for the clip processing pipeline.
# Run from the project root: ./scripts/setup-clipper.sh

set -euo pipefail
cd "$(dirname "$0")/.."

echo "=== GamePilot Clipper Setup ==="
echo ""

# 1. Check Python
echo "[1/5] Checking Python..."
if ! command -v python3 &>/dev/null; then
    echo "ERROR: Python 3 is required but not found."
    echo "  macOS:   brew install python3"
    echo "  Windows: winget install Python.Python.3.12"
    echo "  Linux:   sudo apt install python3 python3-venv"
    exit 1
fi
echo "  Python: $(python3 --version)"

# 2. Check FFmpeg
echo "[2/5] Checking FFmpeg..."
if ! command -v ffmpeg &>/dev/null; then
    echo "WARNING: FFmpeg not found. Clip processing requires it."
    echo "  macOS:   brew install ffmpeg"
    echo "  Windows: winget install ffmpeg"
    echo "  Linux:   sudo apt install ffmpeg"
else
    echo "  FFmpeg: $(ffmpeg -version 2>&1 | head -1)"
fi

# 3. Python venv + dependencies
echo "[3/5] Setting up Python sidecar..."
python3 sidecar/setup.py --deps

# 4. Node dependencies
echo "[4/5] Installing Node dependencies..."
if command -v pnpm &>/dev/null; then
    pnpm install
elif command -v npm &>/dev/null; then
    npm install
else
    echo "ERROR: pnpm or npm required."
    exit 1
fi

# 5. Verify
echo "[5/5] Verifying installation..."
python3 sidecar/setup.py --check

echo ""
echo "=== Setup complete ==="
echo ""
echo "To run in development:"
echo "  pnpm tauri dev"
echo ""
echo "The clipper sidecar starts automatically with the app."
echo "To download ML models (optional, downloads on first use):"
echo "  python3 sidecar/setup.py --models"
echo ""
echo "For Ollama (scoring rationale):"
echo "  ollama pull qwen3:8b"
