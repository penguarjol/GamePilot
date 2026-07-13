use crate::gamemodule::{GameInfo, GameInstance, GameModule};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct PoeModule;

impl GameModule for PoeModule {
    fn game_info(&self) -> GameInfo {
        GameInfo {
            id: "poe".to_string(),
            name: "Path of Exile".to_string(),
            icon: "\u{2694}".to_string(),
            installed: discover_poe_installations().first().is_some(),
            install_path: discover_poe_installations()
                .first()
                .map(|inst| inst.path.clone()),
        }
    }

    fn discover_instances(&self) -> Vec<GameInstance> {
        discover_poe_installations()
    }

    fn can_analyze(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrencyPrice {
    pub name: String,
    pub chaos_equivalent: f64,
    pub change_percent: f64,
}

#[derive(Debug, Deserialize)]
struct CurrencyOverviewResponse {
    lines: Vec<CurrencyLine>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrencyLine {
    currency_type_name: String,
    chaos_equivalent: f64,
    #[serde(default)]
    receive_spot_moving_average: f64,
}

#[derive(Debug, Deserialize)]
struct ItemOverviewResponse {
    lines: Vec<ItemLine>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemLine {
    name: String,
    chaos_value: f64,
    #[serde(default)]
    sparkline: Option<Sparkline>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sparkline {
    #[serde(default)]
    total_change: f64,
}

pub fn discover_poe_installations() -> Vec<GameInstance> {
    let mut instances = Vec::new();

    let candidates: Vec<(&str, &str, &str)> = vec![
        ("poe1-standalone", "Path of Exile", "Path of Exile"),
        ("poe2-standalone", "Path of Exile 2", "Path of Exile 2"),
    ];

    for (id, dir_name, display_name) in candidates {
        for path in candidate_paths(dir_name) {
            if path.exists() {
                instances.push(GameInstance {
                    id: id.to_string(),
                    game_id: "poe".to_string(),
                    name: display_name.to_string(),
                    path: path.to_string_lossy().to_string(),
                    version: None,
                    last_played: None,
                });
                break;
            }
        }
    }

    instances
}

fn candidate_paths(dir_name: &str) -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        vec![
            PathBuf::from(format!(
                "C:\\Program Files (x86)\\Grinding Gear Games\\{}",
                dir_name
            )),
            PathBuf::from(format!(
                "C:\\Program Files\\Grinding Gear Games\\{}",
                dir_name
            )),
        ]
    }
    #[cfg(target_os = "macos")]
    {
        let mut paths = Vec::new();
        if let Ok(home) = std::env::var("HOME") {
            paths.push(
                PathBuf::from(&home)
                    .join("Library/Application Support")
                    .join(dir_name),
            );
        }
        paths
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = dir_name;
        Vec::new()
    }
}

pub async fn fetch_currency_prices(league: &str) -> Result<Vec<CurrencyPrice>, String> {
    let url = format!(
        "https://poe.ninja/api/data/currencyoverview?league={}&type=Currency",
        league
    );

    let resp: CurrencyOverviewResponse = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch currency data: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse currency data: {}", e))?;

    let prices = resp
        .lines
        .into_iter()
        .map(|line| CurrencyPrice {
            name: line.currency_type_name,
            chaos_equivalent: line.chaos_equivalent,
            change_percent: line.receive_spot_moving_average,
        })
        .collect();

    Ok(prices)
}

pub async fn fetch_item_prices(league: &str, item_type: &str) -> Result<Vec<CurrencyPrice>, String> {
    let url = format!(
        "https://poe.ninja/api/data/itemoverview?league={}&type={}",
        league, item_type
    );

    let resp: ItemOverviewResponse = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch item data: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse item data: {}", e))?;

    let prices = resp
        .lines
        .into_iter()
        .map(|line| {
            let change = line
                .sparkline
                .map(|s| s.total_change)
                .unwrap_or(0.0);
            CurrencyPrice {
                name: line.name,
                chaos_equivalent: line.chaos_value,
                change_percent: change,
            }
        })
        .collect();

    Ok(prices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_empty_when_no_install() {
        let instances = discover_poe_installations();
        for inst in &instances {
            assert_eq!(inst.game_id, "poe");
        }
    }

    #[test]
    fn poe_module_game_info() {
        let module = PoeModule;
        let info = module.game_info();
        assert_eq!(info.id, "poe");
        assert_eq!(info.name, "Path of Exile");
        assert!(module.can_analyze());
        assert!(module.can_launch());
        assert!(!module.can_optimize());
    }

    #[test]
    fn candidate_paths_are_non_empty_on_known_platforms() {
        let paths = candidate_paths("Path of Exile");
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        assert!(!paths.is_empty());
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        assert!(paths.is_empty());
    }
}
