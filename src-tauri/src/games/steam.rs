use crate::gamemodule::{GameInfo, GameInstance, GameModule};
use std::path::PathBuf;

pub struct SteamModule;

impl GameModule for SteamModule {
    fn game_info(&self) -> GameInfo {
        GameInfo {
            id: "steam".to_string(),
            name: "Steam".to_string(),
            icon: "\u{2636}".to_string(),
            installed: find_steam_path().is_some(),
            install_path: find_steam_path().map(|p| p.to_string_lossy().to_string()),
        }
    }

    fn discover_instances(&self) -> Vec<GameInstance> {
        discover_steam_games()
    }
}

fn find_steam_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let paths = [
            PathBuf::from("C:\\Program Files (x86)\\Steam"),
            PathBuf::from("C:\\Program Files\\Steam"),
        ];
        paths.into_iter().find(|p| p.exists())
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        let p = PathBuf::from(home).join("Library/Application Support/Steam");
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

fn discover_steam_games() -> Vec<GameInstance> {
    let steam_path = match find_steam_path() {
        Some(p) => p,
        None => return Vec::new(),
    };

    let steamapps = steam_path.join("steamapps");
    let mut games = Vec::new();

    let library_paths = parse_library_folders(&steamapps);

    for lib_path in &library_paths {
        if let Ok(entries) = std::fs::read_dir(lib_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                    if let Some(game) = parse_app_manifest(&entry.path()) {
                        games.push(game);
                    }
                }
            }
        }
    }

    games
}

fn parse_library_folders(steamapps: &PathBuf) -> Vec<PathBuf> {
    let mut paths = vec![steamapps.clone()];
    let vdf_path = steamapps.join("libraryfolders.vdf");

    if let Ok(content) = std::fs::read_to_string(&vdf_path) {
        for line in content.lines() {
            let trimmed = line.trim().trim_matches('"');
            if trimmed.contains("path") {
                if let Some(path_val) = line.split('"').nth(3) {
                    let p = PathBuf::from(path_val.replace("\\\\", "\\")).join("steamapps");
                    if p.exists() {
                        paths.push(p);
                    }
                }
            }
        }
    }

    paths
}

fn parse_app_manifest(path: &PathBuf) -> Option<GameInstance> {
    let content = std::fs::read_to_string(path).ok()?;

    let get_value = |key: &str| -> Option<String> {
        content
            .lines()
            .find(|l| l.trim().starts_with(&format!("\"{}\"", key)))
            .and_then(|l| l.split('"').nth(3))
            .map(String::from)
    };

    let app_id = get_value("appid")?;
    let name = get_value("name")?;
    let install_dir = get_value("installdir")?;

    if name.contains("Redistributable")
        || name.contains("Proton")
        || name.contains("Runtime")
    {
        return None;
    }

    let lib_path = path.parent()?;
    let game_path = lib_path.join("common").join(&install_dir);

    Some(GameInstance {
        id: format!("steam-{}", app_id),
        game_id: "steam".to_string(),
        name,
        path: game_path.to_string_lossy().to_string(),
        version: None,
        last_played: None,
    })
}
