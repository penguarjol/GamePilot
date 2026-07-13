use crate::gamemodule::{GameInfo, GameInstance, GameModule};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct LeagueModule;

impl GameModule for LeagueModule {
    fn game_info(&self) -> GameInfo {
        let install = find_league_install();
        GameInfo {
            id: "league-of-legends".to_string(),
            name: "League of Legends".to_string(),
            icon: "\u{2694}".to_string(),
            installed: install.is_some(),
            install_path: install.map(|p| p.to_string_lossy().to_string()),
        }
    }

    fn discover_instances(&self) -> Vec<GameInstance> {
        let install = match find_league_install() {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![GameInstance {
            id: "league-main".to_string(),
            game_id: "league-of-legends".to_string(),
            name: "League of Legends".to_string(),
            path: install.to_string_lossy().to_string(),
            version: None,
            last_played: None,
        }]
    }

    fn can_optimize(&self) -> bool {
        true
    }
}

fn find_league_install() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            PathBuf::from(r"C:\Riot Games\League of Legends"),
            PathBuf::from(r"C:\Riot Games\Riot Client"),
            PathBuf::from(r"D:\Riot Games\League of Legends"),
        ];
        if let Some(p) = candidates.into_iter().find(|p| p.exists()) {
            return Some(p);
        }

        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let riot_services =
                PathBuf::from(local_app_data).join("Riot Games").join("RiotClientServices");
            if riot_services.exists() {
                let default = PathBuf::from(r"C:\Riot Games\League of Legends");
                return Some(default);
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        let p = PathBuf::from("/Applications/League of Legends.app");
        if p.exists() {
            Some(p)
        } else {
            None
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

// --- Live Client Data API ---
// Runs at https://127.0.0.1:2999/liveclientdata/ during an active game.
// Self-signed cert requires accepting invalid certs for localhost.

const LIVE_CLIENT_BASE: &str = "https://127.0.0.1:2999/liveclientdata";

fn build_live_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub async fn is_game_active() -> bool {
    let client = build_live_client();
    client
        .get(format!("{}/allgamedata", LIVE_CLIENT_BASE))
        .send()
        .await
        .is_ok_and(|r| r.status().is_success())
}

pub async fn get_all_game_data() -> Result<serde_json::Value, String> {
    let client = build_live_client();
    let resp = client
        .get(format!("{}/allgamedata", LIVE_CLIENT_BASE))
        .send()
        .await
        .map_err(|e| format!("Live Client Data unavailable: {}", e))?;

    resp.json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse game data: {}", e))
}

pub async fn get_active_player() -> Result<ActivePlayer, String> {
    let client = build_live_client();
    let resp = client
        .get(format!("{}/activeplayer", LIVE_CLIENT_BASE))
        .send()
        .await
        .map_err(|e| format!("Live Client Data unavailable: {}", e))?;

    resp.json::<ActivePlayer>()
        .await
        .map_err(|e| format!("Failed to parse active player: {}", e))
}

pub async fn get_player_list() -> Result<Vec<Player>, String> {
    let client = build_live_client();
    let resp = client
        .get(format!("{}/playerlist", LIVE_CLIENT_BASE))
        .send()
        .await
        .map_err(|e| format!("Live Client Data unavailable: {}", e))?;

    resp.json::<Vec<Player>>()
        .await
        .map_err(|e| format!("Failed to parse player list: {}", e))
}

pub async fn get_event_data() -> Result<EventData, String> {
    let client = build_live_client();
    let resp = client
        .get(format!("{}/eventdata", LIVE_CLIENT_BASE))
        .send()
        .await
        .map_err(|e| format!("Live Client Data unavailable: {}", e))?;

    resp.json::<EventData>()
        .await
        .map_err(|e| format!("Failed to parse event data: {}", e))
}

// --- Data Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePlayer {
    pub summoner_name: Option<String>,
    pub level: Option<u32>,
    pub current_gold: Option<f64>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub summoner_name: Option<String>,
    pub champion_name: Option<String>,
    pub team: Option<String>,
    pub is_bot: Option<bool>,
    pub scores: Option<PlayerScores>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerScores {
    pub kills: Option<u32>,
    pub deaths: Option<u32>,
    pub assists: Option<u32>,
    pub creep_score: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventData {
    #[serde(rename = "Events")]
    pub events: Option<Vec<GameEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    #[serde(rename = "EventID")]
    pub event_id: Option<u32>,
    #[serde(rename = "EventName")]
    pub event_name: Option<String>,
    #[serde(rename = "EventTime")]
    pub event_time: Option<f64>,
}

// --- Performance Recommendations ---

#[derive(Debug, Clone, Serialize)]
pub struct LeagueRecommendation {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
}

pub fn generate_performance_recommendations() -> Vec<LeagueRecommendation> {
    let mut recs = Vec::new();

    let processes_to_close = [
        ("chrome", "Google Chrome"),
        ("firefox", "Firefox"),
        ("discord", "Discord"),
        ("spotify", "Spotify"),
        ("obs", "OBS Studio"),
    ];

    for (proc_name, display_name) in &processes_to_close {
        if crate::hardware::is_process_running(proc_name) {
            recs.push(LeagueRecommendation {
                id: format!("league-close-{}", proc_name),
                title: format!("Close {}", display_name),
                description: format!(
                    "{} is running and may impact League of Legends performance. \
                     Consider closing it before or during gameplay.",
                    display_name
                ),
                severity: "low".to_string(),
            });
        }
    }

    if !find_league_install().is_some() {
        recs.push(LeagueRecommendation {
            id: "league-not-installed".to_string(),
            title: "League of Legends not found".to_string(),
            description: "Could not detect a League of Legends installation on this system."
                .to_string(),
            severity: "info".to_string(),
        });
    }

    recs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_info_returns_correct_metadata() {
        let module = LeagueModule;
        let info = module.game_info();
        assert_eq!(info.id, "league-of-legends");
        assert_eq!(info.name, "League of Legends");
    }

    #[test]
    fn discover_instances_returns_at_most_one() {
        let module = LeagueModule;
        let instances = module.discover_instances();
        assert!(instances.len() <= 1);
        if let Some(inst) = instances.first() {
            assert_eq!(inst.game_id, "league-of-legends");
        }
    }

    #[test]
    fn recommendations_does_not_panic() {
        let recs = generate_performance_recommendations();
        for rec in &recs {
            assert!(!rec.id.is_empty());
            assert!(!rec.title.is_empty());
        }
    }

    #[tokio::test]
    async fn is_game_active_returns_false_when_no_game() {
        // No League game running during tests — should return false, not panic
        let active = is_game_active().await;
        assert!(!active);
    }
}
