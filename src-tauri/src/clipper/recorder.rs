use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    pub use_builtin: bool,
    pub encoder: String,
    pub quality_preset: String,
    pub replay_buffer_seconds: u32,
    pub hotkey_clip: String,
    pub hotkey_bookmark: String,
    pub storage_limit_gb: u32,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            use_builtin: false,
            encoder: "h264".into(),
            quality_preset: "high".into(),
            replay_buffer_seconds: 30,
            hotkey_clip: "F10".into(),
            hotkey_bookmark: "F8".into(),
            storage_limit_gb: 100,
        }
    }
}
