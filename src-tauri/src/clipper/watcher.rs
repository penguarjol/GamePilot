use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, Serialize)]
pub struct NewRecordingEvent {
    pub path: String,
    pub source: String,
    pub size_bytes: u64,
    pub detected_at: String,
}

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "mov", "avi", "webm", "flv", "ts"];

pub fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn file_is_stable(path: &Path, wait_secs: u64) -> bool {
    let Ok(meta1) = std::fs::metadata(path) else { return false };
    let size1 = meta1.len();
    std::thread::sleep(Duration::from_secs(wait_secs));
    let Ok(meta2) = std::fs::metadata(path) else { return false };
    meta2.len() == size1
}

pub fn start_watcher(
    config: WatcherConfig,
    on_new_recording: impl Fn(NewRecordingEvent) + Send + 'static,
) -> Result<RecommendedWatcher, String> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(
        move |res| { let _ = tx.send(res); },
        Config::default().with_poll_interval(Duration::from_secs(5)),
    ).map_err(|e| format!("Failed to create watcher: {}", e))?;

    let active_folders: Vec<WatchFolder> = config
        .folders
        .into_iter()
        .filter(|f| f.active)
        .collect();

    for folder in &active_folders {
        let path = Path::new(&folder.path);
        if path.exists() {
            watcher.watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| format!("Failed to watch {}: {}", folder.path, e))?;
            log::info!("Watching folder: {} ({})", folder.path, folder.source);
        } else {
            log::warn!("Watch folder does not exist: {}", folder.path);
        }
    }

    let folder_map: std::collections::HashMap<PathBuf, String> = active_folders
        .iter()
        .map(|f| (PathBuf::from(&f.path), f.source.clone()))
        .collect();

    std::thread::spawn(move || {
        let mut recently_seen: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
        let mut last_cleanup = Instant::now();

        for event in rx {
            let Ok(event) = event else { continue };

            if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                continue;
            }

            for path in event.paths {
                if !is_video_file(&path) || recently_seen.contains(&path) {
                    continue;
                }

                if !file_is_stable(&path, 3) {
                    continue;
                }

                recently_seen.insert(path.clone());

                let source = path.parent()
                    .and_then(|p| folder_map.get(p))
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());

                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                log::info!("New recording detected: {} ({})", path.display(), source);

                on_new_recording(NewRecordingEvent {
                    path: path.to_string_lossy().to_string(),
                    source,
                    size_bytes: size,
                    detected_at: chrono::Utc::now().to_rfc3339(),
                });
            }

            if last_cleanup.elapsed() > Duration::from_secs(300) {
                recently_seen.clear();
                last_cleanup = Instant::now();
            }
        }
    });

    Ok(watcher)
}
