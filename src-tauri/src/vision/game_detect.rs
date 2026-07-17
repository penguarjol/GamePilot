use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DetectedGame {
    pub game_id: String,
    pub game_name: String,
    pub process_name: String,
    pub window_title: Option<String>,
    pub is_running: bool,
}

const KNOWN_GAMES: &[(&str, &str, &[&str])] = &[
    ("minecraft", "Minecraft", &["java", "javaw", "minecraft"]),
    (
        "league",
        "League of Legends",
        &["LeagueClient", "League of Legends"],
    ),
    ("valorant", "VALORANT", &["VALORANT-Win64-Shipping"]),
    ("tarkov", "Escape from Tarkov", &["EscapeFromTarkov"]),
    (
        "runescape",
        "RuneScape",
        &["rs2client", "RuneScape", "runelite"],
    ),
    (
        "poe",
        "Path of Exile",
        &["PathOfExile", "PathOfExile_x64", "PathOfExileSteam"],
    ),
    ("csgo", "Counter-Strike", &["cs2", "csgo"]),
    (
        "fortnite",
        "Fortnite",
        &["FortniteClient-Win64-Shipping"],
    ),
    ("overwatch", "Overwatch", &["Overwatch"]),
    ("apex", "Apex Legends", &["r5apex"]),
    ("gta5", "GTA V", &["GTA5", "PlayGTAV"]),
    ("cyberpunk", "Cyberpunk 2077", &["Cyberpunk2077"]),
    ("wow", "World of Warcraft", &["Wow", "WowClassic"]),
    ("diablo4", "Diablo IV", &["Diablo IV"]),
    ("steam_generic", "Steam Game", &[]),
];

/// Detect which game is currently running.
pub fn detect_running_game() -> Option<DetectedGame> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    for (game_id, game_name, process_names) in KNOWN_GAMES {
        if process_names.is_empty() {
            continue;
        }

        for proc in sys.processes().values() {
            let proc_name = proc.name().to_lowercase();
            for pattern in *process_names {
                if proc_name.contains(&pattern.to_lowercase()) {
                    return Some(DetectedGame {
                        game_id: game_id.to_string(),
                        game_name: game_name.to_string(),
                        process_name: proc.name().to_string(),
                        window_title: super::capture::get_foreground_window_title(),
                        is_running: true,
                    });
                }
            }
        }
    }

    None
}

/// Get all currently detected games (there might be multiple).
pub fn detect_all_running_games() -> Vec<DetectedGame> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let mut found = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for (game_id, game_name, process_names) in KNOWN_GAMES {
        if process_names.is_empty() || seen_ids.contains(*game_id) {
            continue;
        }

        for proc in sys.processes().values() {
            let proc_name = proc.name().to_lowercase();
            for pattern in *process_names {
                if proc_name.contains(&pattern.to_lowercase()) && !seen_ids.contains(*game_id) {
                    seen_ids.insert(*game_id);
                    found.push(DetectedGame {
                        game_id: game_id.to_string(),
                        game_name: game_name.to_string(),
                        process_name: proc.name().to_string(),
                        window_title: None,
                        is_running: true,
                    });
                }
            }
        }
    }

    found
}
