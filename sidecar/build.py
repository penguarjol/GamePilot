#!/usr/bin/env python3
"""
Build the GamePilot Clipper sidecar into a single executable via PyInstaller.

Output: dist/clipper-sidecar.exe (Windows) or dist/clipper-sidecar (macOS/Linux)
This binary is then placed into src-tauri/binaries/ for Tauri to bundle.

Usage:
    python3 sidecar/build.py
"""

import platform
import shutil
import subprocess
import sys
from pathlib import Path

SIDECAR_DIR = Path(__file__).parent
PROJECT_ROOT = SIDECAR_DIR.parent
DIST_DIR = SIDECAR_DIR / "dist"
BINARIES_DIR = PROJECT_ROOT / "src-tauri" / "binaries"


def get_target_triple() -> str:
    machine = platform.machine().lower()
    system = platform.system().lower()

    if system == "windows":
        if machine in ("amd64", "x86_64"):
            return "x86_64-pc-windows-msvc"
        return f"{machine}-pc-windows-msvc"
    elif system == "darwin":
        if machine == "arm64":
            return "aarch64-apple-darwin"
        return "x86_64-apple-darwin"
    elif system == "linux":
        return f"{machine}-unknown-linux-gnu"
    return f"{machine}-{system}"


def build():
    print("=== Building Clipper Sidecar ===")

    triple = get_target_triple()
    ext = ".exe" if platform.system() == "Windows" else ""
    output_name = f"clipper-sidecar-{triple}{ext}"

    print(f"Target: {triple}")
    print(f"Output: {output_name}")

    hidden_imports = [
        "uvicorn",
        "uvicorn.logging",
        "uvicorn.protocols",
        "uvicorn.protocols.http",
        "uvicorn.protocols.http.auto",
        "uvicorn.protocols.websockets",
        "uvicorn.protocols.websockets.auto",
        "uvicorn.lifespan",
        "uvicorn.lifespan.on",
        "fastapi",
        "pydantic",
        "detection",
        "detection.audio",
        "detection.scene",
        "detection.whisper_detect",
        "detection.yolo_detect",
        "detection.ocr",
        "detection.composite",
        "processing",
        "processing.clip",
        "processing.effects",
        "processing.recipes",
        "processing.compilation",
    ]

    cmd = [
        sys.executable, "-m", "PyInstaller",
        "--onefile",
        "--name", f"clipper-sidecar-{triple}",
        "--distpath", str(DIST_DIR),
        "--workpath", str(SIDECAR_DIR / "build"),
        "--specpath", str(SIDECAR_DIR),
        "--paths", str(SIDECAR_DIR),
        "--clean",
    ]

    for hi in hidden_imports:
        cmd.extend(["--hidden-import", hi])

    cmd.append(str(SIDECAR_DIR / "api.py"))

    print(f"Running: {' '.join(cmd[:5])}...")
    subprocess.run(cmd, check=True, cwd=str(SIDECAR_DIR))

    BINARIES_DIR.mkdir(parents=True, exist_ok=True)
    src = DIST_DIR / f"clipper-sidecar-{triple}{ext}"
    dst = BINARIES_DIR / output_name

    shutil.copy2(str(src), str(dst))
    print(f"Copied to: {dst}")
    print("=== Build complete ===")


if __name__ == "__main__":
    build()
