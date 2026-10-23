use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub id: String,
    pub clip_id: String,
    pub duration_ms: i64,
    pub output_format: String,
    pub tracks: Vec<Track>,
    pub key_moments: Vec<KeyMoment>,
    pub recipe_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub track_type: String,
    pub segments: Vec<Segment>,
    pub muted: bool,
    pub volume: f64,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub track_id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub source: SegmentSource,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SegmentSource {
    #[serde(rename = "clip")]
    Clip { clip_id: String, source_start_ms: i64, source_end_ms: i64 },
    #[serde(rename = "audio")]
    Audio { path: Option<String>, url: Option<String>, name: String, artist: Option<String> },
    #[serde(rename = "gif")]
    Gif { giphy_id: String, url: String, width: u32, height: u32 },
    #[serde(rename = "sticker")]
    Sticker { giphy_id: String, url: String },
    #[serde(rename = "text")]
    Text { content: String, style: String },
    #[serde(rename = "sfx")]
    Sfx { preset_id: String, name: String },
    #[serde(rename = "image")]
    Image { path: Option<String>, url: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Effect {
    #[serde(rename = "speed")]
    Speed { factor: f64 },
    #[serde(rename = "zoom")]
    Zoom { center_x: f64, center_y: f64, scale: f64 },
    #[serde(rename = "shake")]
    Shake { intensity: f64, duration_ms: i64 },
    #[serde(rename = "voice_mod")]
    VoiceMod { preset: String },
    #[serde(rename = "fade")]
    Fade { direction: String, duration_ms: i64 },
    #[serde(rename = "crop")]
    Crop { mode: String },
    #[serde(rename = "blur_bg")]
    BlurBg,
    #[serde(rename = "opacity")]
    Opacity { value: f64 },
    #[serde(rename = "impact_text")]
    ImpactText { text: String, style: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMoment {
    pub timestamp_ms: i64,
    pub moment_type: String,
    pub label: String,
    pub confidence: f64,
    pub signals: Vec<String>,
}
