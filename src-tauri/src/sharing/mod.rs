use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationProfile {
    pub version: String,
    pub created_at: String,
    pub game: String,
    pub instance_name: String,
    pub minecraft_version: Option<String>,
    pub loader: Option<String>,
    pub jvm_settings: Option<JvmProfile>,
    pub config_changes: Vec<ConfigChange>,
    pub recommended_mods: Vec<RecommendedMod>,
    pub health_score: Option<u32>,
    pub hardware_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmProfile {
    pub xmx_mb: Option<u32>,
    pub xms_mb: Option<u32>,
    pub jvm_args: Option<String>,
    pub java_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub file: String,
    pub key: String,
    pub value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedMod {
    pub name: String,
    pub modrinth_slug: Option<String>,
    pub reason: String,
}

pub fn generate_profile(
    instance: &crate::minecraft::instance::MinecraftInstance,
    recommendations: &[crate::minecraft::rules::Recommendation],
    health_score: Option<u32>,
) -> OptimizationProfile {
    let jvm = JvmProfile {
        xmx_mb: instance.xmx_mb,
        xms_mb: instance.xms_mb,
        jvm_args: instance.jvm_args.clone(),
        java_version: None,
    };

    let recommended_mods: Vec<RecommendedMod> = recommendations
        .iter()
        .filter(|r| r.category == "modpack" && r.action_type.as_deref() == Some("open_link"))
        .map(|r| RecommendedMod {
            name: r.title.clone(),
            modrinth_slug: r.action_data.clone(),
            reason: r.description.clone(),
        })
        .collect();

    OptimizationProfile {
        version: "1.0.0".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        game: "minecraft".to_string(),
        instance_name: instance.name.clone(),
        minecraft_version: instance.minecraft_version.clone(),
        loader: instance.loader_type.clone(),
        jvm_settings: Some(jvm),
        config_changes: Vec::new(),
        recommended_mods,
        health_score,
        hardware_summary: None,
    }
}

pub fn export_profile(profile: &OptimizationProfile) -> Result<String, String> {
    serde_json::to_string_pretty(profile).map_err(|e| format!("Serialize error: {}", e))
}

pub fn import_profile(json: &str) -> Result<OptimizationProfile, String> {
    serde_json::from_str(json).map_err(|e| format!("Invalid profile: {}", e))
}

pub fn format_for_discord(profile: &OptimizationProfile) -> String {
    let mut msg = String::new();
    msg.push_str("**GamePilot Optimization Profile**\n");
    msg.push_str(&format!("Game: {}\n", profile.game));
    msg.push_str(&format!("Instance: {}\n", profile.instance_name));
    if let Some(ver) = &profile.minecraft_version {
        msg.push_str(&format!("Version: {}\n", ver));
    }
    if let Some(loader) = &profile.loader {
        msg.push_str(&format!("Loader: {}\n", loader));
    }
    if let Some(score) = profile.health_score {
        msg.push_str(&format!("Health Score: {}/100\n", score));
    }
    if let Some(jvm) = &profile.jvm_settings {
        if let Some(xmx) = jvm.xmx_mb {
            msg.push_str(&format!("RAM: {} MB\n", xmx));
        }
    }
    if !profile.recommended_mods.is_empty() {
        msg.push_str(&format!(
            "\nRecommended Mods ({}):\n",
            profile.recommended_mods.len()
        ));
        for m in &profile.recommended_mods {
            msg.push_str(&format!("- {}\n", m.name));
        }
    }
    msg
}

pub async fn send_to_discord(
    webhook_url: &str,
    profile: &OptimizationProfile,
) -> Result<(), String> {
    let message = format_for_discord(profile);
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "content": message,
        "username": "GamePilot",
    });

    let resp = client
        .post(webhook_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Discord send failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Discord error: HTTP {}", resp.status()));
    }
    Ok(())
}

pub async fn send_test_to_discord(webhook_url: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "content": "GamePilot webhook connected successfully.",
        "username": "GamePilot",
    });

    let resp = client
        .post(webhook_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Discord send failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Discord error: HTTP {}", resp.status()));
    }
    Ok(())
}
