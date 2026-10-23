"""Speech transcription via faster-whisper with word-level timestamps."""

from typing import Optional

REACTION_WORDS = {
    "clip", "oh my god", "insane", "no way", "what", "holy", "lets go",
    "let's go", "dude", "bro", "nice", "clutch", "ace", "kill", "snipe",
}


async def transcribe_video(
    video_path: str,
    language: Optional[str] = None,
    word_timestamps: bool = True,
) -> dict:
    """Transcribe video audio using faster-whisper, returning word-level timestamps."""
    import asyncio
    return await asyncio.to_thread(
        _transcribe_sync, video_path, language, word_timestamps
    )


def _transcribe_sync(
    video_path: str,
    language: Optional[str],
    word_timestamps: bool,
) -> dict:
    import subprocess
    import tempfile
    from pathlib import Path

    from faster_whisper import WhisperModel

    wav_path = Path(tempfile.mktemp(suffix=".wav"))
    try:
        subprocess.run(
            [
                "ffmpeg", "-i", video_path,
                "-vn", "-acodec", "pcm_s16le", "-ar", "16000", "-ac", "1",
                str(wav_path), "-y",
            ],
            capture_output=True,
            check=True,
        )

        model = WhisperModel("large-v3", device="auto", compute_type="auto")

        segments_iter, info = model.transcribe(
            str(wav_path),
            language=language,
            word_timestamps=word_timestamps,
            vad_filter=True,
        )

        full_text_parts: list[str] = []
        segments_out: list[dict] = []
        words_out: list[dict] = []

        for segment in segments_iter:
            full_text_parts.append(segment.text.strip())
            seg_data = {
                "start": segment.start,
                "end": segment.end,
                "text": segment.text.strip(),
            }
            segments_out.append(seg_data)

            if segment.words:
                for w in segment.words:
                    word_lower = w.word.strip().lower()
                    is_reaction = any(r in word_lower for r in REACTION_WORDS)
                    words_out.append({
                        "word": w.word.strip(),
                        "start": w.start,
                        "end": w.end,
                        "probability": round(w.probability, 3),
                        "is_reaction": is_reaction,
                    })

        return {
            "text": " ".join(full_text_parts),
            "segments": segments_out,
            "words": words_out,
            "language": info.language,
        }
    finally:
        wav_path.unlink(missing_ok=True)
