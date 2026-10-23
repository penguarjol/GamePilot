"""OCR-based event confirmation — filters killcam/spectating false positives."""


FILTER_KEYWORDS = [
    "killcam", "spectating", "replay", "final kill", "play of the game",
    "best play", "potg", "death cam", "spectator",
]


def filter_false_positives(candidates: list[dict], video_path: str) -> list[dict]:
    """Remove candidates that occur during killcam or spectating sequences."""
    import cv2
    import easyocr

    if not candidates:
        return candidates

    reader = easyocr.Reader(["en"], gpu=True)
    cap = cv2.VideoCapture(video_path)
    fps = cap.get(cv2.CAP_PROP_FPS) or 30.0
    filtered = []

    for candidate in candidates:
        ts_ms = candidate.get("timestamp_ms", 0)
        frame_num = int((ts_ms / 1000.0) * fps)
        cap.set(cv2.CAP_PROP_POS_FRAMES, frame_num)
        ret, frame = cap.read()

        if not ret:
            filtered.append(candidate)
            continue

        h, w = frame.shape[:2]
        roi = frame[0 : h // 4, :]
        results = reader.readtext(roi, detail=0)
        text_joined = " ".join(results).lower()

        is_false_positive = any(kw in text_joined for kw in FILTER_KEYWORDS)

        if not is_false_positive:
            filtered.append(candidate)

    cap.release()
    return filtered
