"""Audio peak detection via librosa."""

from pathlib import Path


def detect_audio_peaks(video_path: str, threshold_db: float = -20.0) -> dict:
    """Detect audio peaks (loud moments) in a video file."""
    import subprocess
    import tempfile

    import librosa
    import numpy as np

    wav_path = Path(tempfile.mktemp(suffix=".wav"))
    try:
        subprocess.run(
            [
                "ffmpeg", "-i", video_path,
                "-vn", "-acodec", "pcm_s16le", "-ar", "22050", "-ac", "1",
                str(wav_path), "-y",
            ],
            capture_output=True,
            check=True,
        )

        y, sr = librosa.load(str(wav_path), sr=22050, mono=True)
        duration_ms = int(len(y) / sr * 1000)

        frame_length = 2048
        hop_length = 512
        rms = librosa.feature.rms(y=y, frame_length=frame_length, hop_length=hop_length)[0]

        rms_db = librosa.amplitude_to_db(rms, ref=np.max)
        times = librosa.frames_to_time(range(len(rms_db)), sr=sr, hop_length=hop_length)

        median_db = float(np.median(rms_db))
        peak_threshold = max(threshold_db, median_db + 10)

        peaks = []
        min_gap_s = 2.0
        last_peak_time = -min_gap_s

        for i, (t, db_val) in enumerate(zip(times, rms_db)):
            if db_val > peak_threshold and (t - last_peak_time) >= min_gap_s:
                peaks.append({
                    "timestamp_ms": int(t * 1000),
                    "type": "audio_peak",
                    "label": f"Audio peak ({db_val:.1f} dB)",
                    "confidence": float(min(1.0, (db_val - median_db) / 20.0)),
                    "signal": "audio",
                })
                last_peak_time = t

        return {"peaks": peaks, "duration_ms": duration_ms}

    finally:
        wav_path.unlink(missing_ok=True)
