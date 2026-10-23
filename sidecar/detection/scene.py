"""Scene change detection via PySceneDetect."""


def detect_scene_changes(video_path: str, threshold: float = 27.0) -> dict:
    """Detect scene boundaries in a video using ContentDetector."""
    from scenedetect import ContentDetector, open_video, SceneManager

    video = open_video(video_path)
    scene_manager = SceneManager()
    scene_manager.add_detector(ContentDetector(threshold=threshold))
    scene_manager.detect_scenes(video)

    scene_list = scene_manager.get_scene_list()
    scenes = []
    for start, end in scene_list:
        scenes.append({
            "timestamp_ms": int(start.get_seconds() * 1000),
            "end_ms": int(end.get_seconds() * 1000),
            "type": "scene_change",
            "label": f"Scene change at {start.get_timecode()}",
            "confidence": 0.8,
            "signal": "scene",
        })

    return {"scenes": scenes}
