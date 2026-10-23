use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub weight_audio: f64,
    pub weight_scene: f64,
    pub weight_yolo: f64,
    pub weight_whisper: f64,
    pub weight_vlm: f64,
    pub weight_trend: f64,
    pub decima_prompt: String,
}

impl MoodProfile {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: "skill".into(),
                name: "Skill".into(),
                description: "Mechanical precision, multi-kills, clutches, headshots".into(),
                weight_audio: 0.10,
                weight_scene: 0.10,
                weight_yolo: 0.35,
                weight_whisper: 0.05,
                weight_vlm: 0.30,
                weight_trend: 0.10,
                decima_prompt: "Rate this clip's mechanical skill display and competitive impressiveness.".into(),
            },
            Self {
                id: "funny".into(),
                name: "Funny".into(),
                description: "Reactions, unexpected outcomes, fails, absurd situations".into(),
                weight_audio: 0.25,
                weight_scene: 0.10,
                weight_yolo: 0.10,
                weight_whisper: 0.25,
                weight_vlm: 0.20,
                weight_trend: 0.10,
                decima_prompt: "Rate how funny, surprising, or entertaining this moment is.".into(),
            },
            Self {
                id: "montage".into(),
                name: "Montage".into(),
                description: "Clean kills with rhythm, back-to-back action, visual flow".into(),
                weight_audio: 0.15,
                weight_scene: 0.15,
                weight_yolo: 0.30,
                weight_whisper: 0.05,
                weight_vlm: 0.25,
                weight_trend: 0.10,
                decima_prompt: "Rate this clip's visual rhythm, pacing, and suitability for a highlight reel.".into(),
            },
            Self {
                id: "viral".into(),
                name: "Viral".into(),
                description: "Hook in first 2 seconds, emotional peak, relatability".into(),
                weight_audio: 0.20,
                weight_scene: 0.10,
                weight_yolo: 0.15,
                weight_whisper: 0.15,
                weight_vlm: 0.25,
                weight_trend: 0.15,
                decima_prompt: "Rate this clip's viral potential: hook strength, emotional impact, shareability.".into(),
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub game_id: String,
    pub profile_id: String,
    pub decima_tau: f64,
    pub weights: MoodProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipScore {
    pub id: String,
    pub clip_id: String,
    pub scorer: String,
    pub score_level: String,
    pub confidence: f64,
    pub escalated: bool,
    pub rationale: Option<String>,
}
