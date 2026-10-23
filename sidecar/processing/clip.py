"""FFmpeg-based clip extraction and formatting."""

import asyncio
import subprocess
from pathlib import Path
from typing import Optional


async def extract_clip(
    video_path: str,
    start_ms: int,
    end_ms: int,
    output_path: Optional[str] = None,
    fmt: str = "mp4",
    vertical_crop: bool = False,
    burn_captions: bool = False,
) -> dict:
    """Extract a clip from a video using FFmpeg."""
    return await asyncio.to_thread(
        _extract_sync, video_path, start_ms, end_ms, output_path, fmt, vertical_crop, burn_captions
    )


def _extract_sync(
    video_path: str,
    start_ms: int,
    end_ms: int,
    output_path: Optional[str],
    fmt: str,
    vertical_crop: bool,
    burn_captions: bool,
) -> dict:
    src = Path(video_path)
    if output_path is None:
        output_path = str(src.parent / f"{src.stem}_clip_{start_ms}_{end_ms}.{fmt}")

    start_s = start_ms / 1000.0
    duration_s = (end_ms - start_ms) / 1000.0

    vf_filters: list[str] = []
    if vertical_crop:
        vf_filters.append("crop=ih*9/16:ih:(iw-ih*9/16)/2:0")

    cmd = [
        "ffmpeg",
        "-ss", f"{start_s:.3f}",
        "-i", video_path,
        "-t", f"{duration_s:.3f}",
        "-c:v", "libx264",
        "-preset", "fast",
        "-crf", "18",
        "-c:a", "aac",
        "-b:a", "192k",
    ]

    if vf_filters:
        cmd.extend(["-vf", ",".join(vf_filters)])

    cmd.extend(["-y", output_path])

    subprocess.run(cmd, capture_output=True, check=True)

    return {
        "output_path": output_path,
        "duration_ms": end_ms - start_ms,
        "format": fmt,
    }
