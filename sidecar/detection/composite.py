"""
Composite detection — runs multiple detection signals in sequence and fuses results.
"""

import asyncio
from pathlib import Path
from typing import Optional


async def run_detection(
    video_path: str,
    game_id: Optional[str] = None,
    signals: list[str] | None = None,
    frame_skip: int = 10,
) -> dict:
    """Run requested detection signals on a video and return fused candidates."""
    signals = signals or ["audio", "scene", "whisper"]
    path = Path(video_path)

    if not path.exists():
        raise FileNotFoundError(f"Video not found: {video_path}")

    candidates: list[dict] = []
    signals_used: list[str] = []
    duration_ms = 0

    if "audio" in signals:
        from .audio import detect_audio_peaks
        audio_result = await asyncio.to_thread(detect_audio_peaks, str(path))
        candidates.extend(audio_result.get("peaks", []))
        duration_ms = audio_result.get("duration_ms", 0)
        signals_used.append("audio")

    if "scene" in signals:
        from .scene import detect_scene_changes
        scene_result = await asyncio.to_thread(detect_scene_changes, str(path))
        candidates.extend(scene_result.get("scenes", []))
        signals_used.append("scene")

    if "whisper" in signals:
        from .whisper_detect import transcribe_video
        whisper_result = await transcribe_video(str(path))
        for word in whisper_result.get("words", []):
            if word.get("is_reaction", False):
                candidates.append({
                    "timestamp_ms": int(word["start"] * 1000),
                    "type": "reaction",
                    "label": word.get("word", ""),
                    "confidence": word.get("probability", 0.5),
                    "signal": "whisper",
                })
        signals_used.append("whisper")

    if "yolo" in signals:
        from .yolo_detect import detect_game_events
        yolo_result = await asyncio.to_thread(
            detect_game_events, str(path), game_id=game_id, frame_skip=frame_skip
        )
        candidates.extend(yolo_result.get("events", []))
        signals_used.append("yolo")

    if "ocr" in signals:
        from .ocr import filter_false_positives
        candidates = await asyncio.to_thread(filter_false_positives, candidates, str(path))
        signals_used.append("ocr")

    candidates.sort(key=lambda c: c.get("timestamp_ms", 0))

    return {
        "video_path": video_path,
        "duration_ms": duration_ms,
        "candidates": candidates,
        "signals_used": signals_used,
    }
