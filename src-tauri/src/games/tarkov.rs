use crate::gamemodule::{GameInfo, GameInstance, GameModule};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct TarkovModule;

impl GameModule for TarkovModule {
    fn game_info(&self) -> GameInfo {
        let install = find_tarkov_path();
        GameInfo {
            id: "tarkov".to_string(),
            name: "Escape from Tarkov".to_string(),
            icon: "\u{2694}".to_string(),
            installed: install.is_some(),
            install_path: install.map(|p| p.to_string_lossy().to_string()),
        }
    }

    fn discover_instances(&self) -> Vec<GameInstance> {
        let Some(path) = find_tarkov_path() else {
            return Vec::new();
        };
        vec![GameInstance {
            id: "tarkov-live".to_string(),
            game_id: "tarkov".to_string(),
            name: "Escape from Tarkov".to_string(),
            path: path.to_string_lossy().to_string(),
            version: None,
            last_played: None,
        }]
    }

    fn can_analyze(&self) -> bool {
        true
    }
}

fn find_tarkov_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            PathBuf::from(r"C:\Battlestate Games\EFT"),
            PathBuf::from(r"C:\Battlestate Games\Escape From Tarkov"),
            PathBuf::from(r"C:\Battlestate Games\BsgLauncher"),
        ];
        candidates.into_iter().find(|p| p.exists())
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

const TARKOV_API: &str = "https://api.tarkov.dev/graphql";

// --- Public data types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmoData {
    pub name: String,
    pub short_name: String,
    pub caliber: String,
    pub damage: i32,
    pub penetration: i32,
    pub armor_damage: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPrice {
    pub name: String,
    pub short_name: String,
    pub avg_24h_price: i64,
    pub last_low_price: i64,
}

// --- GraphQL response shapes ---

#[derive(Deserialize)]
struct GqlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize)]
struct GqlError {
    message: String,
}

#[derive(Deserialize)]
struct AmmoRoot {
    ammo: Vec<AmmoNode>,
}

#[derive(Deserialize)]
struct AmmoNode {
    item: AmmoItemRef,
    caliber: Option<String>,
    damage: Option<i32>,
    #[serde(rename = "penetrationPower")]
    penetration_power: Option<i32>,
    #[serde(rename = "armorDamage")]
    armor_damage: Option<i32>,
}

#[derive(Deserialize)]
struct AmmoItemRef {
    name: Option<String>,
    #[serde(rename = "shortName")]
    short_name: Option<String>,
}

#[derive(Deserialize)]
struct ItemsRoot {
    items: Vec<ItemNode>,
}

#[derive(Deserialize)]
struct ItemNode {
    name: Option<String>,
    #[serde(rename = "shortName")]
    short_name: Option<String>,
    #[serde(rename = "avg24hPrice")]
    avg_24h_price: Option<i64>,
    #[serde(rename = "lastLowPrice")]
    last_low_price: Option<i64>,
}

// --- API functions ---

pub async fn fetch_ammo_data() -> Result<Vec<AmmoData>, String> {
    let query = r#"{ ammo { item { name shortName } caliber damage penetrationPower armorDamage } }"#;

    let body = serde_json::json!({ "query": query });

    let resp: GqlResponse<AmmoRoot> = reqwest::Client::new()
        .post(TARKOV_API)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Tarkov API request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse Tarkov API response: {}", e))?;

    if let Some(errors) = resp.errors {
        let msgs: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
        return Err(format!("Tarkov API errors: {}", msgs.join("; ")));
    }

    let ammo_list = resp
        .data
        .ok_or_else(|| "Tarkov API returned no data".to_string())?
        .ammo;

    Ok(ammo_list
        .into_iter()
        .map(|a| AmmoData {
            name: a.item.name.unwrap_or_default(),
            short_name: a.item.short_name.unwrap_or_default(),
            caliber: a.caliber.unwrap_or_default(),
            damage: a.damage.unwrap_or(0),
            penetration: a.penetration_power.unwrap_or(0),
            armor_damage: a.armor_damage.unwrap_or(0),
        })
        .collect())
}

pub async fn search_items(name: &str) -> Result<Vec<ItemPrice>, String> {
    let query = r#"query($name: String!) { items(name: $name) { name shortName avg24hPrice lastLowPrice } }"#;

    let body = serde_json::json!({
        "query": query,
        "variables": { "name": name },
    });

    let resp: GqlResponse<ItemsRoot> = reqwest::Client::new()
        .post(TARKOV_API)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Tarkov API request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse Tarkov API response: {}", e))?;

    if let Some(errors) = resp.errors {
        let msgs: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
        return Err(format!("Tarkov API errors: {}", msgs.join("; ")));
    }

    let items = resp
        .data
        .ok_or_else(|| "Tarkov API returned no data".to_string())?
        .items;

    Ok(items
        .into_iter()
        .map(|i| ItemPrice {
            name: i.name.unwrap_or_default(),
            short_name: i.short_name.unwrap_or_default(),
            avg_24h_price: i.avg_24h_price.unwrap_or(0),
            last_low_price: i.last_low_price.unwrap_or(0),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_info_has_correct_id() {
        let module = TarkovModule;
        let info = module.game_info();
        assert_eq!(info.id, "tarkov");
        assert_eq!(info.name, "Escape from Tarkov");
    }

    #[test]
    fn can_analyze_returns_true() {
        let module = TarkovModule;
        assert!(module.can_analyze());
    }

    #[test]
    fn ammo_data_serializes() {
        let ammo = AmmoData {
            name: "M80".to_string(),
            short_name: "M80".to_string(),
            caliber: "7.62x51".to_string(),
            damage: 80,
            penetration: 41,
            armor_damage: 52,
        };
        let json = serde_json::to_string(&ammo).unwrap();
        assert!(json.contains("M80"));
        assert!(json.contains("\"damage\":80"));
    }

    #[test]
    fn item_price_serializes() {
        let item = ItemPrice {
            name: "LEDX".to_string(),
            short_name: "LEDX".to_string(),
            avg_24h_price: 1_200_000,
            last_low_price: 1_100_000,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("LEDX"));
        assert!(json.contains("1200000"));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn discover_instances_empty_on_non_windows() {
        let module = TarkovModule;
        assert!(module.discover_instances().is_empty());
    }
}
