"""Highlight reel / montage compilation assembly."""

import subprocess
import tempfile
from pathlib import Path
from typing import Optional

from .effects import apply_transition


def compile_clips(
    clip_paths: list[str],
    output_path: str,
    transition_type: str = "fade",
    transition_duration: float = 0.5,
) -> str:
    """Concatenate multiple clips into a compilation with transitions."""
    if len(clip_paths) == 0:
        raise ValueError("No clips to compile")

    if len(clip_paths) == 1:
        import shutil
        shutil.copy2(clip_paths[0], output_path)
        return output_path

    if transition_type == "cut":
        return _concat_no_transition(clip_paths, output_path)

    current = clip_paths[0]
    for i, next_clip in enumerate(clip_paths[1:], 1):
        tmp_out = tempfile.mktemp(suffix=".mp4")
        apply_transition(current, next_clip, tmp_out, transition_type, transition_duration)
        if i > 1:
            Path(current).unlink(missing_ok=True)
        current = tmp_out

    import shutil
    shutil.move(current, output_path)
    return output_path


def _concat_no_transition(clip_paths: list[str], output_path: str) -> str:
    """Simple concatenation without transitions."""
    list_file = Path(tempfile.mktemp(suffix=".txt"))
    try:
        with open(list_file, "w") as f:
            for p in clip_paths:
                f.write(f"file '{p}'\n")

        cmd = [
            "ffmpeg", "-f", "concat", "-safe", "0",
            "-i", str(list_file),
            "-c", "copy", "-y", output_path,
        ]
        subprocess.run(cmd, capture_output=True, check=True)
        return output_path
    finally:
        list_file.unlink(missing_ok=True)
