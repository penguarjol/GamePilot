"""Auto-effects engine — applies effects at key moments via FFmpeg filter chains."""

import subprocess
from pathlib import Path
from typing import Optional


VOICE_PRESETS = {
    "deep": "rubberband=pitch=0.8:formant=preserved",
    "chipmunk": "rubberband=pitch=1.4:formant=preserved",
    "echo": "aecho=0.8:0.88:60:0.4",
    "reverb": "aecho=0.8:0.9:1000|1800:0.3|0.25",
    "slowmo": "atempo=0.7,rubberband=pitch=1.43:formant=preserved",
}

CAPTION_STYLES = {
    "bold": "fontcolor=white:fontsize=24:borderw=2:bordercolor=black",
    "bar": "fontcolor=white:fontsize=20:box=1:boxcolor=black@0.7:boxborderw=8",
    "karaoke": "fontcolor=yellow:fontsize=22:borderw=1:bordercolor=black",
    "hormozi": "fontcolor=white:fontsize=36:borderw=3:bordercolor=black",
}


def apply_zoom(input_path: str, output_path: str, timestamp_s: float, duration_s: float = 1.0, scale: float = 1.3) -> str:
    """Apply a zoom punch-in effect at a specific timestamp."""
    center_x = 0.5
    center_y = 0.5
    vf = (
        f"zoompan=z='if(between(in_time,{timestamp_s},{timestamp_s + duration_s}),{scale},1)'"
        f":x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d=1:s=1920x1080:fps=30"
    )
    cmd = ["ffmpeg", "-i", input_path, "-vf", vf, "-c:a", "copy", "-y", output_path]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path


def apply_speed_change(input_path: str, output_path: str, start_s: float, end_s: float, factor: float = 0.5) -> str:
    """Apply slow-motion or speed-up to a segment."""
    pts_factor = 1.0 / factor
    vf = f"setpts={pts_factor}*PTS"
    af = f"atempo={factor}" if 0.5 <= factor <= 2.0 else f"atempo={min(2.0, max(0.5, factor))}"
    cmd = ["ffmpeg", "-i", input_path, "-ss", str(start_s), "-to", str(end_s), "-vf", vf, "-af", af, "-y", output_path]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path


def apply_voice_mod(input_path: str, output_path: str, preset: str) -> str:
    """Apply a voice modification preset."""
    af = VOICE_PRESETS.get(preset)
    if not af:
        raise ValueError(f"Unknown voice preset: {preset}")
    cmd = ["ffmpeg", "-i", input_path, "-af", af, "-c:v", "copy", "-y", output_path]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path


def burn_captions(input_path: str, output_path: str, srt_path: str, style: str = "bold") -> str:
    """Burn subtitles onto the video."""
    style_str = CAPTION_STYLES.get(style, CAPTION_STYLES["bold"])
    vf = f"subtitles={srt_path}:force_style='{style_str}'"
    cmd = ["ffmpeg", "-i", input_path, "-vf", vf, "-c:a", "copy", "-y", output_path]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path


def apply_transition(clip_a: str, clip_b: str, output_path: str, transition_type: str = "fade", duration_s: float = 0.5) -> str:
    """Apply a transition between two clips using xfade."""
    import json
    probe = subprocess.run(
        ["ffprobe", "-v", "quiet", "-print_format", "json", "-show_format", clip_a],
        capture_output=True, text=True,
    )
    dur_a = float(json.loads(probe.stdout)["format"]["duration"])
    offset = dur_a - duration_s

    cmd = [
        "ffmpeg",
        "-i", clip_a, "-i", clip_b,
        "-filter_complex", f"[0:v][1:v]xfade=transition={transition_type}:duration={duration_s}:offset={offset}[v];[0:a][1:a]acrossfade=d={duration_s}[a]",
        "-map", "[v]", "-map", "[a]",
        "-y", output_path,
    ]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path


def generate_thumbnail(video_path: str, timestamp_s: float, output_path: str, width: int = 320) -> str:
    """Extract a single frame as a thumbnail."""
    cmd = [
        "ffmpeg", "-ss", str(timestamp_s), "-i", video_path,
        "-vframes", "1", "-vf", f"scale={width}:-1",
        "-y", output_path,
    ]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path


def generate_sprite_sheet(video_path: str, output_path: str, fps: float = 1.0, thumb_width: int = 160, columns: int = 10) -> str:
    """Generate a thumbnail sprite sheet for the filmstrip timeline."""
    vf = f"fps={fps},scale={thumb_width}:-1,tile={columns}x1"
    cmd = ["ffmpeg", "-i", video_path, "-vf", vf, "-frames:v", "1", "-y", output_path]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path


def mix_audio_with_ducking(video_path: str, audio_path: str, output_path: str, duck_level: float = 0.3) -> str:
    """Mix trending audio over the clip with original audio ducked."""
    cmd = [
        "ffmpeg",
        "-i", video_path, "-i", audio_path,
        "-filter_complex",
        f"[0:a]volume={duck_level}[orig];[1:a]volume=1.0[music];[orig][music]amix=inputs=2:duration=first:dropout_transition=2[aout]",
        "-map", "0:v", "-map", "[aout]",
        "-c:v", "copy", "-y", output_path,
    ]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path
