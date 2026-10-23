use crate::clipper::sidecar::SidecarState;

pub async fn process_recording(
    sidecar: &SidecarState,
    video_path: &str,
    game_id: Option<&str>,
    signals: Vec<String>,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "video_path": video_path,
        "game_id": game_id,
        "signals": signals,
    });

    crate::clipper::sidecar::sidecar_request(sidecar, "POST", "/detect", Some(body)).await
}

pub async fn extract_clip(
    sidecar: &SidecarState,
    video_path: &str,
    start_ms: i64,
    end_ms: i64,
    output_path: Option<&str>,
    vertical_crop: bool,
    burn_captions: bool,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "video_path": video_path,
        "start_ms": start_ms,
        "end_ms": end_ms,
        "output_path": output_path,
        "vertical_crop": vertical_crop,
        "burn_captions": burn_captions,
    });

    crate::clipper::sidecar::sidecar_request(sidecar, "POST", "/process", Some(body)).await
}
