# GamePilot Clipper — One-command setup for Windows.
# Run from the project root: .\scripts\setup-clipper.ps1

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot)

Write-Host "=== GamePilot Clipper Setup ===" -ForegroundColor Cyan
Write-Host ""

# 1. Check Python
Write-Host "[1/5] Checking Python..."
$py = Get-Command python3 -ErrorAction SilentlyContinue
if (-not $py) { $py = Get-Command python -ErrorAction SilentlyContinue }
if (-not $py) {
    Write-Host "ERROR: Python 3 required. Install: winget install Python.Python.3.12" -ForegroundColor Red
    exit 1
}
& $py.Source --version

# 2. Check FFmpeg
Write-Host "[2/5] Checking FFmpeg..."
$ff = Get-Command ffmpeg -ErrorAction SilentlyContinue
if (-not $ff) {
    Write-Host "WARNING: FFmpeg not found. Install: winget install ffmpeg" -ForegroundColor Yellow
} else {
    ffmpeg -version 2>&1 | Select-Object -First 1
}

# 3. Python sidecar
Write-Host "[3/5] Setting up Python sidecar..."
& $py.Source sidecar/setup.py --deps

# 4. Node dependencies
Write-Host "[4/5] Installing Node dependencies..."
if (Get-Command pnpm -ErrorAction SilentlyContinue) {
    pnpm install
} elseif (Get-Command npm -ErrorAction SilentlyContinue) {
    npm install
} else {
    Write-Host "ERROR: pnpm or npm required." -ForegroundColor Red
    exit 1
}

# 5. Verify
Write-Host "[5/5] Verifying..."
& $py.Source sidecar/setup.py --check

Write-Host ""
Write-Host "=== Setup complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "To run: pnpm tauri dev"
Write-Host "For scoring rationale: ollama pull qwen3:8b"
