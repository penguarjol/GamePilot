"""
GamePilot Clipper Sidecar — FastAPI server for ML-powered clip detection and processing.

Runs on localhost with an ephemeral port. Tauri manages the lifecycle.
Authentication via per-launch token passed as a CLI argument.
"""

import argparse
import json
import os
import signal
import sys
import time
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Optional

import uvicorn
from fastapi import Depends, FastAPI, Header, HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field


LAUNCH_TOKEN: Optional[str] = None
PARENT_PID: Optional[int] = None


# ── Request / Response Models ────────────────────────────────────────────────

class HealthResponse(BaseModel):
    status: str = "ready"
    version: str = "0.1.0"
    pid: int = 0
    uptime_s: float = 0.0
    models_loaded: list[str] = []


class DetectRequest(BaseModel):
    video_path: str
    game_id: Optional[str] = None
    signals: list[str] = Field(
        default=["audio", "scene", "whisper"],
        description="Which detection signals to run: audio, scene, whisper, yolo, ocr, vlm",
    )
    frame_skip: int = Field(default=10, ge=1, le=100)


class DetectResponse(BaseModel):
    video_path: str
    duration_ms: int = 0
    candidates: list[dict] = []
    signals_used: list[str] = []
    processing_time_s: float = 0.0


class ProcessRequest(BaseModel):
    video_path: str
    start_ms: int
    end_ms: int
    output_path: Optional[str] = None
    format: str = "mp4"
    vertical_crop: bool = False
    burn_captions: bool = False


class ProcessResponse(BaseModel):
    output_path: str
    duration_ms: int = 0
    format: str = "mp4"


class TranscribeRequest(BaseModel):
    video_path: str
    language: Optional[str] = None
    word_timestamps: bool = True


class TranscribeResponse(BaseModel):
    text: str = ""
    segments: list[dict] = []
    words: list[dict] = []
    language: str = ""
    processing_time_s: float = 0.0


# ── Auth ─────────────────────────────────────────────────────────────────────

async def verify_token(authorization: Optional[str] = Header(None)):
    if LAUNCH_TOKEN is None:
        return
    if authorization != LAUNCH_TOKEN:
        raise HTTPException(status_code=401, detail="Invalid launch token")


# ── Parent Process Watchdog ──────────────────────────────────────────────────

_start_time = time.time()


def _check_parent_alive():
    """Exit if the Tauri parent process is gone."""
    if PARENT_PID is None:
        return
    try:
        os.kill(PARENT_PID, 0)
    except OSError:
        print(f"[sidecar] Parent PID {PARENT_PID} is gone, shutting down.", file=sys.stderr)
        os._exit(0)


# ── App ──────────────────────────────────────────────────────────────────────

@asynccontextmanager
async def lifespan(app: FastAPI):
    import asyncio

    async def parent_watchdog():
        while True:
            _check_parent_alive()
            await asyncio.sleep(2)

    task = asyncio.create_task(parent_watchdog())
    yield
    task.cancel()


app = FastAPI(title="GamePilot Clipper Sidecar", version="0.1.0", lifespan=lifespan)


@app.get("/health", response_model=HealthResponse)
async def health():
    return HealthResponse(
        status="ready",
        pid=os.getpid(),
        uptime_s=round(time.time() - _start_time, 2),
        models_loaded=[],
    )


@app.post("/detect", response_model=DetectResponse, dependencies=[Depends(verify_token)])
async def detect(req: DetectRequest):
    t0 = time.time()
    from detection.composite import run_detection

    result = await run_detection(
        video_path=req.video_path,
        game_id=req.game_id,
        signals=req.signals,
        frame_skip=req.frame_skip,
    )
    result["processing_time_s"] = round(time.time() - t0, 3)
    return DetectResponse(**result)


@app.post("/process", response_model=ProcessResponse, dependencies=[Depends(verify_token)])
async def process_clip(req: ProcessRequest):
    from processing.clip import extract_clip

    result = await extract_clip(
        video_path=req.video_path,
        start_ms=req.start_ms,
        end_ms=req.end_ms,
        output_path=req.output_path,
        fmt=req.format,
        vertical_crop=req.vertical_crop,
        burn_captions=req.burn_captions,
    )
    return ProcessResponse(**result)


@app.post("/transcribe", response_model=TranscribeResponse, dependencies=[Depends(verify_token)])
async def transcribe(req: TranscribeRequest):
    t0 = time.time()
    from detection.whisper_detect import transcribe_video

    result = await transcribe_video(
        video_path=req.video_path,
        language=req.language,
        word_timestamps=req.word_timestamps,
    )
    result["processing_time_s"] = round(time.time() - t0, 3)
    return TranscribeResponse(**result)


# ── Entrypoint ───────────────────────────────────────────────────────────────

def main():
    global LAUNCH_TOKEN, PARENT_PID

    parser = argparse.ArgumentParser(description="GamePilot Clipper Sidecar")
    parser.add_argument("--port", type=int, default=0, help="Port to bind (0 = ephemeral)")
    parser.add_argument("--token", type=str, default=None, help="Per-launch auth token")
    parser.add_argument("--parent-pid", type=int, default=None, help="Tauri parent PID for watchdog")
    args = parser.parse_args()

    LAUNCH_TOKEN = args.token
    PARENT_PID = args.parent_pid

    import socket
    if args.port == 0:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.bind(("127.0.0.1", 0))
        port = sock.getsockname()[1]
        sock.close()
    else:
        port = args.port

    handshake = json.dumps({
        "port": port,
        "token": LAUNCH_TOKEN or "",
        "pid": os.getpid(),
        "version": "0.1.0",
    })
    print(handshake, flush=True)

    uvicorn.run(
        app,
        host="127.0.0.1",
        port=port,
        log_level="warning",
        access_log=False,
    )


if __name__ == "__main__":
    main()
