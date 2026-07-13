use crate::gamemodule::{GameInfo, GameInstance, GameModule};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// --- Data Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub username: String,
    pub game: String,
    pub skills: Vec<SkillLevel>,
    pub total_level: u32,
    pub total_xp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLevel {
    pub name: String,
    pub level: u32,
    pub xp: u64,
    pub rank: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrandExchangeItem {
    pub item_id: u32,
    pub name: String,
    pub description: String,
    pub current_price: String,
    pub icon: String,
    pub members: bool,
}

// --- Skill Ordering (matches Jagex Hiscores CSV row order) ---

const OSRS_SKILLS: &[&str] = &[
    "Overall", "Attack", "Defence", "Strength", "Hitpoints", "Ranged",
    "Prayer", "Magic", "Cooking", "Woodcutting", "Fletching", "Fishing",
    "Firemaking", "Crafting", "Smithing", "Mining", "Herblore", "Agility",
    "Thieving", "Slayer", "Farming", "Runecrafting", "Hunter", "Construction",
];

const RS3_SKILLS: &[&str] = &[
    "Overall", "Attack", "Defence", "Strength", "Constitution", "Ranged",
    "Prayer", "Magic", "Cooking", "Woodcutting", "Fletching", "Fishing",
    "Firemaking", "Crafting", "Smithing", "Mining", "Herblore", "Agility",
    "Thieving", "Slayer", "Farming", "Runecrafting", "Hunter", "Construction",
    "Summoning", "Dungeoneering", "Divination", "Invention", "Archaeology",
    "Necromancy",
];

// --- GameModule: RS3 ---

pub struct Rs3Module;

impl GameModule for Rs3Module {
    fn game_info(&self) -> GameInfo {
        let path = find_rs3_path();
        GameInfo {
            id: "runescape-rs3".to_string(),
            name: "RuneScape (RS3)".to_string(),
            icon: "\u{2694}".to_string(),
            installed: path.is_some(),
            install_path: path.map(|p| p.to_string_lossy().to_string()),
        }
    }

    fn discover_instances(&self) -> Vec<GameInstance> {
        let mut instances = Vec::new();
        if let Some(path) = find_rs3_path() {
            instances.push(GameInstance {
                id: "rs3-main".to_string(),
                game_id: "runescape-rs3".to_string(),
                name: "RuneScape".to_string(),
                path: path.to_string_lossy().to_string(),
                version: None,
                last_played: None,
            });
        }
        instances
    }

    fn can_analyze(&self) -> bool {
        true
    }
}

// --- GameModule: OSRS ---

pub struct OsrsModule;

impl GameModule for OsrsModule {
    fn game_info(&self) -> GameInfo {
        let path = find_osrs_path();
        GameInfo {
            id: "runescape-osrs".to_string(),
            name: "Old School RuneScape".to_string(),
            icon: "\u{2694}".to_string(),
            installed: path.is_some(),
            install_path: path.map(|p| p.to_string_lossy().to_string()),
        }
    }

    fn discover_instances(&self) -> Vec<GameInstance> {
        let mut instances = Vec::new();

        if let Some(path) = find_osrs_path() {
            instances.push(GameInstance {
                id: "osrs-jagex".to_string(),
                game_id: "runescape-osrs".to_string(),
                name: "Old School RuneScape (Jagex Launcher)".to_string(),
                path: path.to_string_lossy().to_string(),
                version: None,
                last_played: None,
            });
        }

        if let Some(path) = find_runelite_path() {
            instances.push(GameInstance {
                id: "osrs-runelite".to_string(),
                game_id: "runescape-osrs".to_string(),
                name: "Old School RuneScape (RuneLite)".to_string(),
                path: path.to_string_lossy().to_string(),
                version: None,
                last_played: None,
            });
        }

        instances
    }

    fn can_analyze(&self) -> bool {
        true
    }
}

// --- Installation Discovery ---

fn find_rs3_path() -> Option<PathBuf> {
    candidate_paths_rs3().into_iter().find(|p| p.exists())
}

fn find_osrs_path() -> Option<PathBuf> {
    candidate_paths_osrs().into_iter().find(|p| p.exists())
}

fn find_runelite_path() -> Option<PathBuf> {
    candidate_paths_runelite().into_iter().find(|p| p.exists())
}

fn candidate_paths_rs3() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        paths.push(PathBuf::from(r"C:\Program Files\Jagex\RuneScape"));
        paths.push(PathBuf::from(r"C:\Program Files\Jagex\RuneScape Launcher"));
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            paths.push(PathBuf::from(&local).join("Jagex").join("RuneScape"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Applications/RuneScape.app"));
    }

    paths
}

fn candidate_paths_osrs() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        paths.push(PathBuf::from(r"C:\Program Files\Jagex\Old School RuneScape"));
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            paths.push(
                PathBuf::from(&local)
                    .join("Jagex")
                    .join("Old School RuneScape"),
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Applications/Old School RuneScape.app"));
    }

    paths
}

fn candidate_paths_runelite() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            paths.push(PathBuf::from(&home).join(".runelite"));
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            paths.push(PathBuf::from(&appdata).join("runelite"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            paths.push(
                PathBuf::from(&home)
                    .join("Library")
                    .join("Application Support")
                    .join("runelite"),
            );
            paths.push(PathBuf::from(&home).join(".runelite"));
        }
    }

    paths
}

// --- Hiscores API ---

fn hiscores_url(game: &str, username: &str) -> Result<String, String> {
    let encoded = urlencoded(username);
    match game {
        "osrs" => Ok(format!(
            "https://secure.runescape.com/m=hiscore_oldschool/index_lite.ws?player={}",
            encoded
        )),
        "rs3" => Ok(format!(
            "https://secure.runescape.com/m=hiscore/index_lite.ws?player={}",
            encoded
        )),
        _ => Err(format!("Unknown game variant: {}", game)),
    }
}

fn skills_for_game(game: &str) -> &'static [&'static str] {
    match game {
        "rs3" => RS3_SKILLS,
        _ => OSRS_SKILLS,
    }
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' => {
                c.to_string()
            }
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}

/// Parse the Jagex Hiscores CSV response into `PlayerStats`.
///
/// Each line is `rank,level,xp` (skills) followed by activity rows
/// which we skip. Lines where rank == -1 mean the player is unranked
/// in that skill.
pub fn parse_hiscores_csv(csv: &str, username: &str, game: &str) -> Result<PlayerStats, String> {
    let skill_names = skills_for_game(game);
    let lines: Vec<&str> = csv.lines().collect();

    if lines.len() < skill_names.len() {
        return Err(format!(
            "Unexpected hiscores response: got {} lines, expected at least {}",
            lines.len(),
            skill_names.len()
        ));
    }

    let mut skills = Vec::with_capacity(skill_names.len());

    for (i, &name) in skill_names.iter().enumerate() {
        let parts: Vec<&str> = lines[i].split(',').collect();
        if parts.len() < 3 {
            return Err(format!("Malformed hiscores line {}: '{}'", i, lines[i]));
        }
        let rank: i64 = parts[0].trim().parse().unwrap_or(-1);
        let level: u32 = parts[1].trim().parse().unwrap_or(1);
        let xp: u64 = parts[2].trim().parse().unwrap_or(0);

        skills.push(SkillLevel {
            name: name.to_string(),
            level,
            xp,
            rank: if rank < 0 { 0 } else { rank as u64 },
        });
    }

    let total_level = skills.first().map(|s| s.level).unwrap_or(0);
    let total_xp = skills.first().map(|s| s.xp).unwrap_or(0);

    Ok(PlayerStats {
        username: username.to_string(),
        game: game.to_string(),
        skills,
        total_level,
        total_xp,
    })
}

pub async fn lookup_player(username: &str, game: &str) -> Result<PlayerStats, String> {
    let url = hiscores_url(game, username)?;
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("Hiscores request failed: {}", e))?;

    if resp.status() == 404 {
        return Err(format!("Player '{}' not found on {} hiscores", username, game));
    }

    let csv = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read hiscores response: {}", e))?;

    parse_hiscores_csv(&csv, username, game)
}

// --- Grand Exchange API ---

pub async fn lookup_ge_price(item_id: u32) -> Result<GrandExchangeItem, String> {
    let url = format!(
        "https://services.runescape.com/m=itemdb_oldschool/api/catalogue/detail.json?item={}",
        item_id
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("GE request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GE API returned status {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GE response: {}", e))?;

    let item = body
        .get("item")
        .ok_or_else(|| "Missing 'item' key in GE response".to_string())?;

    Ok(GrandExchangeItem {
        item_id,
        name: item["name"].as_str().unwrap_or("").to_string(),
        description: item["description"].as_str().unwrap_or("").to_string(),
        current_price: item["current"]["price"]
            .as_str()
            .or_else(|| item["current"]["price"].as_i64().map(|_| ""))
            .unwrap_or("")
            .to_string(),
        icon: item["icon"].as_str().unwrap_or("").to_string(),
        members: item["members"].as_str() == Some("true"),
    })
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OSRS_CSV: &str = "\
456,2277,4600000000
12345,99,200000000
23456,99,200000000
34567,99,200000000
45678,99,200000000
56789,99,200000000
67890,99,200000000
78901,99,200000000
89012,99,200000000
90123,99,200000000
11111,99,200000000
22222,99,200000000
33333,99,200000000
44444,99,200000000
55555,99,200000000
66666,99,200000000
77777,99,200000000
88888,99,200000000
99999,99,200000000
10101,99,200000000
20202,99,200000000
30303,99,200000000
40404,99,200000000
50505,99,200000000";

    #[test]
    fn parse_valid_osrs_hiscores() {
        let stats = parse_hiscores_csv(SAMPLE_OSRS_CSV, "Zezima", "osrs").unwrap();
        assert_eq!(stats.username, "Zezima");
        assert_eq!(stats.game, "osrs");
        assert_eq!(stats.skills.len(), OSRS_SKILLS.len());
        assert_eq!(stats.total_level, 2277);
        assert_eq!(stats.total_xp, 4_600_000_000);
        assert_eq!(stats.skills[0].name, "Overall");
        assert_eq!(stats.skills[0].rank, 456);
        assert_eq!(stats.skills[1].name, "Attack");
        assert_eq!(stats.skills[1].level, 99);
    }

    #[test]
    fn parse_unranked_player_has_zero_rank() {
        let csv = SAMPLE_OSRS_CSV.replace("12345,99,200000000", "-1,1,0");
        let stats = parse_hiscores_csv(&csv, "Noob", "osrs").unwrap();
        assert_eq!(stats.skills[1].rank, 0);
        assert_eq!(stats.skills[1].level, 1);
        assert_eq!(stats.skills[1].xp, 0);
    }

    #[test]
    fn parse_too_few_lines_returns_error() {
        let csv = "1,1000,100000\n2,50,5000";
        let result = parse_hiscores_csv(csv, "Short", "osrs");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected at least"));
    }

    #[test]
    fn parse_malformed_line_returns_error() {
        let mut lines: Vec<String> = SAMPLE_OSRS_CSV.lines().map(String::from).collect();
        lines[3] = "bad_data".to_string();
        let csv = lines.join("\n");
        let result = parse_hiscores_csv(&csv, "BadData", "osrs");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Malformed"));
    }

    #[test]
    fn rs3_skills_has_more_entries_than_osrs() {
        assert!(RS3_SKILLS.len() > OSRS_SKILLS.len());
    }

    #[test]
    fn hiscores_url_osrs() {
        let url = hiscores_url("osrs", "Iron Mammal").unwrap();
        assert!(url.contains("hiscore_oldschool"));
        assert!(url.contains("Iron+Mammal"));
    }

    #[test]
    fn hiscores_url_rs3() {
        let url = hiscores_url("rs3", "Zezima").unwrap();
        assert!(url.contains("m=hiscore/"));
        assert!(url.contains("Zezima"));
    }

    #[test]
    fn hiscores_url_unknown_game_returns_error() {
        assert!(hiscores_url("runescape4", "Test").is_err());
    }

    #[test]
    fn urlencoded_handles_spaces_and_specials() {
        assert_eq!(urlencoded("Iron Mammal"), "Iron+Mammal");
        assert_eq!(urlencoded("abc123"), "abc123");
        assert_eq!(urlencoded("a&b"), "a%26b");
    }

    #[test]
    fn osrs_module_game_info_has_correct_id() {
        let module = OsrsModule;
        let info = module.game_info();
        assert_eq!(info.id, "runescape-osrs");
        assert!(info.name.contains("Old School"));
    }

    #[test]
    fn rs3_module_game_info_has_correct_id() {
        let module = Rs3Module;
        let info = module.game_info();
        assert_eq!(info.id, "runescape-rs3");
        assert!(info.name.contains("RS3"));
    }

    #[test]
    fn both_modules_can_analyze() {
        assert!(OsrsModule.can_analyze());
        assert!(Rs3Module.can_analyze());
    }
}
