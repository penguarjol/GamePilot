use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchFolder {
    pub path: String,
    pub source: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub folders: Vec<WatchFolder>,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            folders: vec![
                WatchFolder {
                    path: r"C:\Medal\".to_string(),
                    source: "Medal.tv".to_string(),
                    active: true,
                },
            ],
        }
    }
}

pub fn is_video_file(path: &std::path::Path) -> bool {
    let extensions = ["mp4", "mkv", "mov", "avi", "webm", "flv"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}
