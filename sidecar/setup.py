#!/usr/bin/env python3
"""
GamePilot Clipper Sidecar — Setup and model download script.

Run once after cloning to set up the Python environment and download required models.
Usage:
    python3 sidecar/setup.py           # Full setup: venv + deps + models
    python3 sidecar/setup.py --deps    # Dependencies only
    python3 sidecar/setup.py --models  # Model download only
    python3 sidecar/setup.py --check   # Verify everything is installed
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path

SIDECAR_DIR = Path(__file__).parent
VENV_DIR = SIDECAR_DIR / ".venv"
MODELS_DIR = SIDECAR_DIR / "models"
REQUIREMENTS = SIDECAR_DIR / "requirements.txt"


def run(cmd: list[str], **kwargs):
    print(f"  > {' '.join(cmd)}")
    subprocess.run(cmd, check=True, **kwargs)


def python_bin() -> str:
    if VENV_DIR.exists():
        if sys.platform == "win32":
            return str(VENV_DIR / "Scripts" / "python.exe")
        return str(VENV_DIR / "bin" / "python3")
    return sys.executable


def pip_bin() -> str:
    if VENV_DIR.exists():
        if sys.platform == "win32":
            return str(VENV_DIR / "Scripts" / "pip.exe")
        return str(VENV_DIR / "bin" / "pip")
    return "pip3"


def create_venv():
    if VENV_DIR.exists():
        print("[venv] Already exists, skipping.")
        return
    print("[venv] Creating virtual environment...")
    run([sys.executable, "-m", "venv", str(VENV_DIR)])
    print("[venv] Created.")


def install_deps():
    print("[deps] Installing Python dependencies...")
    run([pip_bin(), "install", "--upgrade", "pip"])
    run([pip_bin(), "install", "-r", str(REQUIREMENTS)])
    print("[deps] Done.")


def download_models():
    MODELS_DIR.mkdir(exist_ok=True)

    whisper_marker = MODELS_DIR / ".whisper_ready"
    if not whisper_marker.exists():
        print("[models] Pre-downloading faster-whisper large-v3 (this may take a few minutes)...")
        script = (
            "from faster_whisper import WhisperModel; "
            "m = WhisperModel('large-v3', device='cpu', compute_type='int8'); "
            "print('Whisper large-v3 ready')"
        )
        try:
            run([python_bin(), "-c", script])
            whisper_marker.write_text("ok")
        except subprocess.CalledProcessError:
            print("[models] Whisper download failed. It will download on first use instead.")
    else:
        print("[models] Whisper large-v3 already downloaded.")

    print("[models] YOLO and other models download on first use per game.")
    print("[models] Decima ONNX: download from https://huggingface.co/amyrmahdy/decima-small")

    readme = MODELS_DIR / "README.md"
    if not readme.exists():
        readme.write_text(
            "# Models\n\n"
            "Place model files here. They are downloaded automatically on first use.\n\n"
            "- `yolo_*.pt` — YOLO v11 weights per game (from NiceShot AI, VIDCOM, etc.)\n"
            "- Whisper large-v3 — downloaded by faster-whisper to its cache dir\n"
            "- Decima ONNX — download from HuggingFace: amyrmahdy/decima-small\n"
            "- Qwen3-8B — pulled via Ollama: `ollama pull qwen3:8b`\n"
            "- Qwen2.5-VL-3B — pulled via Ollama or HuggingFace transformers cache\n"
        )


def check_ffmpeg():
    print("[ffmpeg] Checking FFmpeg installation...")
    try:
        result = subprocess.run(["ffmpeg", "-version"], capture_output=True, text=True)
        version_line = result.stdout.split("\n")[0] if result.stdout else "unknown"
        print(f"[ffmpeg] Found: {version_line}")
        return True
    except FileNotFoundError:
        print("[ffmpeg] NOT FOUND. Install FFmpeg:")
        print("  Windows: winget install ffmpeg")
        print("  macOS:   brew install ffmpeg")
        print("  Linux:   sudo apt install ffmpeg")
        return False


def check_ollama():
    print("[ollama] Checking Ollama installation...")
    try:
        result = subprocess.run(["ollama", "list"], capture_output=True, text=True)
        print(f"[ollama] Found. Models: {result.stdout.strip()[:200]}")
        return True
    except FileNotFoundError:
        print("[ollama] NOT FOUND. Install from https://ollama.com")
        print("  Then run: ollama pull qwen3:8b")
        return False


def check_all():
    print("=== GamePilot Clipper Sidecar — Dependency Check ===\n")

    ok = True

    print("[python] Version:", sys.version.split()[0])

    if VENV_DIR.exists():
        print(f"[venv] Found at {VENV_DIR}")
    else:
        print("[venv] NOT FOUND. Run: python3 sidecar/setup.py")
        ok = False

    try:
        run([python_bin(), "-c", "import fastapi; import librosa; import scenedetect; import faster_whisper; print('Core deps OK')"])
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("[deps] Missing Python packages. Run: python3 sidecar/setup.py --deps")
        ok = False

    if not check_ffmpeg():
        ok = False

    check_ollama()

    if ok:
        print("\n=== All checks passed. Sidecar is ready. ===")
    else:
        print("\n=== Some checks failed. See above for fixes. ===")

    return ok


def main():
    parser = argparse.ArgumentParser(description="GamePilot Clipper Sidecar Setup")
    parser.add_argument("--deps", action="store_true", help="Install Python deps only")
    parser.add_argument("--models", action="store_true", help="Download models only")
    parser.add_argument("--check", action="store_true", help="Check installation status")
    args = parser.parse_args()

    if args.check:
        sys.exit(0 if check_all() else 1)

    if args.deps:
        create_venv()
        install_deps()
        return

    if args.models:
        download_models()
        return

    print("=== GamePilot Clipper Sidecar — Full Setup ===\n")
    create_venv()
    install_deps()
    check_ffmpeg()
    download_models()
    check_ollama()
    print("\n=== Setup complete. Run 'python3 sidecar/setup.py --check' to verify. ===")


if __name__ == "__main__":
    main()
