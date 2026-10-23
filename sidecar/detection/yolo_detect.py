"""YOLO v11 event detection for game-specific highlights."""

from pathlib import Path
from typing import Optional


MODEL_REGISTRY: dict[str, str] = {
    "valorant": "models/yolo_valorant.pt",
    "cs2": "models/yolo_cs2.pt",
    "fortnite": "models/yolo_fortnite.pt",
    "overwatch": "models/yolo_overwatch.pt",
    "apex": "models/yolo_apex.pt",
    "cod": "models/yolo_cod.pt",
    "generic": "models/yolo_generic.pt",
}


def detect_game_events(
    video_path: str,
    game_id: Optional[str] = None,
    frame_skip: int = 10,
    confidence_threshold: float = 0.5,
) -> dict:
    """Run YOLO event detection on a video for a specific game."""
    from ultralytics import YOLO
    import cv2

    model_key = game_id or "generic"
    model_path = MODEL_REGISTRY.get(model_key)

    if model_path is None or not Path(model_path).exists():
        return {"events": [], "error": f"No model for game: {model_key}"}

    model = YOLO(model_path)
    cap = cv2.VideoCapture(video_path)
    fps = cap.get(cv2.CAP_PROP_FPS) or 30.0
    events: list[dict] = []
    frame_idx = 0

    while cap.isOpened():
        ret, frame = cap.read()
        if not ret:
            break

        if frame_idx % frame_skip == 0:
            results = model(frame, verbose=False)
            for r in results:
                for box in r.boxes:
                    conf = float(box.conf[0])
                    if conf >= confidence_threshold:
                        cls_id = int(box.cls[0])
                        cls_name = model.names.get(cls_id, f"class_{cls_id}")
                        timestamp_ms = int((frame_idx / fps) * 1000)
                        events.append({
                            "timestamp_ms": timestamp_ms,
                            "type": _classify_event(cls_name),
                            "label": cls_name,
                            "confidence": round(conf, 3),
                            "signal": "yolo",
                            "bbox": box.xyxy[0].tolist(),
                        })

        frame_idx += 1

    cap.release()
    return {"events": _deduplicate_events(events)}


def _classify_event(class_name: str) -> str:
    """Map YOLO class names to standardized event types."""
    name = class_name.lower()
    if any(k in name for k in ["kill", "elimination", "frag"]):
        return "kill"
    if any(k in name for k in ["death", "died", "knocked"]):
        return "death"
    if any(k in name for k in ["headshot", "head"]):
        return "kill"
    if any(k in name for k in ["ace", "multi", "triple", "double", "quad"]):
        return "multi_kill"
    if any(k in name for k in ["assist"]):
        return "assist"
    return "event"


def _deduplicate_events(events: list[dict], min_gap_ms: int = 2000) -> list[dict]:
    """Remove duplicate events that fire within min_gap_ms of each other."""
    if not events:
        return events

    events.sort(key=lambda e: e["timestamp_ms"])
    deduped = [events[0]]

    for evt in events[1:]:
        last = deduped[-1]
        if (evt["timestamp_ms"] - last["timestamp_ms"]) >= min_gap_ms or evt["type"] != last["type"]:
            deduped.append(evt)
        elif evt["confidence"] > last["confidence"]:
            deduped[-1] = evt

    return deduped
