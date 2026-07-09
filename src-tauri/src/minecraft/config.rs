use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ConfigAnalysis {
    pub options: Option<OptionsAnalysis>,
    pub server_properties: Option<ServerPropertiesAnalysis>,
    pub recommendations: Vec<ConfigRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptionsAnalysis {
    pub render_distance: Option<i32>,
    pub simulation_distance: Option<i32>,
    pub max_framerate: Option<i32>,
    pub graphics_level: Option<String>,
    pub gui_scale: Option<i32>,
    pub vsync: Option<bool>,
    pub entity_shadows: Option<bool>,
    pub fullscreen: Option<bool>,
    pub raw_entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerPropertiesAnalysis {
    pub view_distance: Option<i32>,
    pub simulation_distance: Option<i32>,
    pub max_players: Option<i32>,
    pub spawn_protection: Option<i32>,
    pub max_tick_time: Option<i64>,
    pub network_compression_threshold: Option<i32>,
    pub raw_entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigRecommendation {
    pub file: String,
    pub key: String,
    pub current_value: String,
    pub recommended_value: String,
    pub reason: String,
    pub impact: String,
    pub confidence: String,
}

pub fn analyze_configs(instance_path: &Path, mod_count: usize) -> ConfigAnalysis {
    let mc_dir = find_mc_dir(instance_path);
    let options = parse_options_txt(&mc_dir);
    let server_props = parse_server_properties(&mc_dir);
    let recommendations = generate_config_recommendations(&options, &server_props, mod_count);

    ConfigAnalysis {
        options,
        server_properties: server_props,
        recommendations,
    }
}

fn find_mc_dir(path: &Path) -> std::path::PathBuf {
    if path.join(".minecraft").exists() {
        path.join(".minecraft")
    } else if path.join("minecraft").exists() {
        path.join("minecraft")
    } else {
        path.to_path_buf()
    }
}

fn parse_key_value_file(path: &Path) -> Option<HashMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':').or_else(|| line.split_once('=')) {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    Some(map)
}

fn parse_options_txt(mc_dir: &Path) -> Option<OptionsAnalysis> {
    let path = mc_dir.join("options.txt");
    let entries = parse_key_value_file(&path)?;

    Some(OptionsAnalysis {
        render_distance: entries.get("renderDistance").and_then(|v| v.parse().ok()),
        simulation_distance: entries.get("simulationDistance").and_then(|v| v.parse().ok()),
        max_framerate: entries.get("maxFps").and_then(|v| v.parse().ok()),
        graphics_level: entries.get("graphicsMode").cloned(),
        gui_scale: entries.get("guiScale").and_then(|v| v.parse().ok()),
        vsync: entries.get("enableVsync").map(|v| v == "true"),
        entity_shadows: entries.get("entityShadows").map(|v| v == "true"),
        fullscreen: entries.get("fullscreen").map(|v| v == "true"),
        raw_entries: entries,
    })
}

fn parse_server_properties(mc_dir: &Path) -> Option<ServerPropertiesAnalysis> {
    let path = mc_dir.join("server.properties");
    let entries = parse_key_value_file(&path)?;

    Some(ServerPropertiesAnalysis {
        view_distance: entries.get("view-distance").and_then(|v| v.parse().ok()),
        simulation_distance: entries
            .get("simulation-distance")
            .and_then(|v| v.parse().ok()),
        max_players: entries.get("max-players").and_then(|v| v.parse().ok()),
        spawn_protection: entries.get("spawn-protection").and_then(|v| v.parse().ok()),
        max_tick_time: entries.get("max-tick-time").and_then(|v| v.parse().ok()),
        network_compression_threshold: entries
            .get("network-compression-threshold")
            .and_then(|v| v.parse().ok()),
        raw_entries: entries,
    })
}

fn generate_config_recommendations(
    options: &Option<OptionsAnalysis>,
    server_props: &Option<ServerPropertiesAnalysis>,
    mod_count: usize,
) -> Vec<ConfigRecommendation> {
    let mut recs = Vec::new();
    let is_heavy_pack = mod_count > 100;

    if let Some(opts) = options {
        if let Some(rd) = opts.render_distance {
            if is_heavy_pack && rd > 12 {
                recs.push(ConfigRecommendation {
                    file: "options.txt".to_string(),
                    key: "renderDistance".to_string(),
                    current_value: rd.to_string(),
                    recommended_value: "10".to_string(),
                    reason: format!(
                        "Render distance {} is high for a {} mod pack. \
                         Reducing to 10 significantly decreases chunk rendering load.",
                        rd, mod_count
                    ),
                    impact: "Medium — fewer chunks rendered, smoother FPS".to_string(),
                    confidence: "high".to_string(),
                });
            }
        }

        if let Some(sd) = opts.simulation_distance {
            if is_heavy_pack && sd > 8 {
                recs.push(ConfigRecommendation {
                    file: "options.txt".to_string(),
                    key: "simulationDistance".to_string(),
                    current_value: sd.to_string(),
                    recommended_value: "6".to_string(),
                    reason: format!(
                        "Simulation distance {} causes more entities and tile entities to tick. \
                         Reducing to 6 is the largest single-setting performance gain for heavy packs.",
                        sd
                    ),
                    impact: "High — fewer ticking entities, lower CPU load".to_string(),
                    confidence: "high".to_string(),
                });
            }
        }

        if let Some(fps) = opts.max_framerate {
            if fps == 260 || fps >= 999 {
                recs.push(ConfigRecommendation {
                    file: "options.txt".to_string(),
                    key: "maxFps".to_string(),
                    current_value: fps.to_string(),
                    recommended_value: "120".to_string(),
                    reason: "Unlimited framerate causes unnecessary GPU load and heat. \
                             Capping at your monitor's refresh rate or 120 is sufficient."
                        .to_string(),
                    impact: "Low — reduced GPU power draw and heat".to_string(),
                    confidence: "medium".to_string(),
                });
            }
        }
    }

    if let Some(sp) = server_props {
        if let Some(vd) = sp.view_distance {
            if is_heavy_pack && vd > 10 {
                recs.push(ConfigRecommendation {
                    file: "server.properties".to_string(),
                    key: "view-distance".to_string(),
                    current_value: vd.to_string(),
                    recommended_value: "8".to_string(),
                    reason: format!(
                        "Server view distance {} is high for a heavy modpack. \
                         Reducing to 8 cuts chunk generation and network traffic.",
                        vd
                    ),
                    impact: "High — less server CPU and network usage".to_string(),
                    confidence: "high".to_string(),
                });
            }
        }

        if let Some(sd) = sp.simulation_distance {
            if is_heavy_pack && sd > 8 {
                recs.push(ConfigRecommendation {
                    file: "server.properties".to_string(),
                    key: "simulation-distance".to_string(),
                    current_value: sd.to_string(),
                    recommended_value: "6".to_string(),
                    reason: format!(
                        "Server simulation distance {} causes excessive entity ticking. \
                         6-8 is recommended for heavy modpacks.",
                        sd
                    ),
                    impact: "High — significant TPS improvement".to_string(),
                    confidence: "high".to_string(),
                });
            }
        }
    }

    recs
}
