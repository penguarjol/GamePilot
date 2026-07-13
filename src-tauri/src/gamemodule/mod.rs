use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GameInfo {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub installed: bool,
    pub install_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameInstance {
    pub id: String,
    pub game_id: String,
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub last_played: Option<String>,
}

/// Capability trait that all game modules implement.
pub trait GameModule: Send + Sync {
    fn game_info(&self) -> GameInfo;
    fn discover_instances(&self) -> Vec<GameInstance>;
    fn can_launch(&self) -> bool {
        true
    }
    fn can_analyze(&self) -> bool {
        false
    }
    fn can_optimize(&self) -> bool {
        false
    }
}
