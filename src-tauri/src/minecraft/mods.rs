use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ModInfo {
    pub file_name: String,
    pub mod_id: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModAnalysis {
    pub total_mods: usize,
    pub mods: Vec<ModInfo>,
    pub detected_performance_mods: Vec<String>,
    pub missing_performance_mods: Vec<PerformanceModRecommendation>,
    pub total_size_mb: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceModRecommendation {
    pub mod_name: String,
    pub mod_id: String,
    pub reason: String,
    pub expected_impact: String,
    pub confidence: String,
    pub url: String,
    pub loaders: Vec<String>,
}

const KNOWN_PERFORMANCE_MODS: &[(&str, &str)] = &[
    ("modernfix", "ModernFix"),
    ("ferritecore", "FerriteCore"),
    ("sodium", "Sodium"),
    ("embeddium", "Embeddium"),
    ("entityculling", "Entity Culling"),
    ("entity_culling", "Entity Culling"),
    ("immediatelyfast", "ImmediatelyFast"),
    ("servercore", "ServerCore"),
    ("lithium", "Lithium"),
    ("starlight", "Starlight"),
    ("lazydfu", "LazyDFU"),
    ("smoothboot", "Smooth Boot"),
    ("dynamic_fps", "Dynamic FPS"),
    ("dynamicfps", "Dynamic FPS"),
    ("fastsuite", "FastSuite"),
    ("noisium", "Noisium"),
    ("clumps", "Clumps"),
    ("krypton", "Krypton"),
    ("memoryleakfix", "MemoryLeakFix"),
    ("moreculling", "More Culling"),
    ("exordium", "Exordium"),
    ("distanthorizons", "Distant Horizons"),
    ("distant_horizons", "Distant Horizons"),
];

fn recommended_performance_mods() -> Vec<PerformanceModRecommendation> {
    vec![
        PerformanceModRecommendation {
            mod_name: "ModernFix".to_string(),
            mod_id: "modernfix".to_string(),
            reason: "Reduces memory usage and improves startup time. Compatible with most modpacks.".to_string(),
            expected_impact: "Medium — reduced memory usage, faster startup".to_string(),
            confidence: "high".to_string(),
            url: "https://modrinth.com/mod/modernfix".to_string(),
            loaders: vec!["Forge".into(), "NeoForge".into(), "Fabric".into()],
        },
        PerformanceModRecommendation {
            mod_name: "FerriteCore".to_string(),
            mod_id: "ferritecore".to_string(),
            reason: "Significantly reduces memory usage through data structure optimization.".to_string(),
            expected_impact: "High — 50-200MB RAM reduction".to_string(),
            confidence: "high".to_string(),
            url: "https://modrinth.com/mod/ferrite-core".to_string(),
            loaders: vec!["Forge".into(), "NeoForge".into(), "Fabric".into()],
        },
        PerformanceModRecommendation {
            mod_name: "Entity Culling".to_string(),
            mod_id: "entityculling".to_string(),
            reason: "Skips rendering entities that are not visible, improving FPS.".to_string(),
            expected_impact: "Medium — +10-30% FPS in entity-heavy areas".to_string(),
            confidence: "high".to_string(),
            url: "https://modrinth.com/mod/entityculling".to_string(),
            loaders: vec!["Forge".into(), "NeoForge".into(), "Fabric".into()],
        },
        PerformanceModRecommendation {
            mod_name: "ImmediatelyFast".to_string(),
            mod_id: "immediatelyfast".to_string(),
            reason: "Optimizes immediate-mode rendering, improving HUD and GUI performance.".to_string(),
            expected_impact: "Low-Medium — smoother UI and HUD rendering".to_string(),
            confidence: "high".to_string(),
            url: "https://modrinth.com/mod/immediatelyfast".to_string(),
            loaders: vec!["Forge".into(), "NeoForge".into(), "Fabric".into()],
        },
        PerformanceModRecommendation {
            mod_name: "Clumps".to_string(),
            mod_id: "clumps".to_string(),
            reason: "Merges XP orbs into single entities, reducing entity count.".to_string(),
            expected_impact: "Low — reduces entity-related lag".to_string(),
            confidence: "high".to_string(),
            url: "https://modrinth.com/mod/clumps".to_string(),
            loaders: vec!["Forge".into(), "NeoForge".into(), "Fabric".into()],
        },
    ]
}

pub fn analyze_mods(mods_path: &Path, loader: Option<&str>) -> ModAnalysis {
    let mut mods = Vec::new();
    let mut total_size: u64 = 0;

    if let Ok(entries) = std::fs::read_dir(mods_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

            if !file_name.ends_with(".jar") && !file_name.ends_with(".zip") {
                continue;
            }

            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            total_size += size;

            mods.push(ModInfo {
                file_name: file_name.clone(),
                mod_id: extract_mod_id(&file_name),
                display_name: None,
                version: extract_version(&file_name),
                size_bytes: size,
            });
        }
    }

    let detected_performance_mods = detect_performance_mods(&mods);
    let missing_performance_mods = find_missing_performance_mods(&detected_performance_mods, loader);

    ModAnalysis {
        total_mods: mods.len(),
        mods,
        detected_performance_mods,
        missing_performance_mods,
        total_size_mb: total_size as f64 / (1024.0 * 1024.0),
    }
}

fn detect_performance_mods(mods: &[ModInfo]) -> Vec<String> {
    let mut detected = Vec::new();

    for mod_info in mods {
        let lower = mod_info.file_name.to_lowercase();
        for (pattern, name) in KNOWN_PERFORMANCE_MODS {
            if lower.contains(pattern) && !detected.contains(&name.to_string()) {
                detected.push(name.to_string());
            }
        }
    }

    detected
}

fn find_missing_performance_mods(
    detected: &[String],
    loader: Option<&str>,
) -> Vec<PerformanceModRecommendation> {
    recommended_performance_mods()
        .into_iter()
        .filter(|rec| {
            let already_installed = detected.iter().any(|d| d.to_lowercase() == rec.mod_name.to_lowercase());
            if already_installed {
                return false;
            }
            if let Some(loader) = loader {
                rec.loaders.iter().any(|l| l.to_lowercase() == loader.to_lowercase())
            } else {
                true
            }
        })
        .collect()
}

fn extract_mod_id(filename: &str) -> Option<String> {
    let name = filename
        .trim_end_matches(".jar")
        .trim_end_matches(".zip");
    let parts: Vec<&str> = name.splitn(2, '-').collect();
    if !parts.is_empty() {
        Some(parts[0].to_lowercase())
    } else {
        None
    }
}

fn extract_version(filename: &str) -> Option<String> {
    let name = filename
        .trim_end_matches(".jar")
        .trim_end_matches(".zip");
    let parts: Vec<&str> = name.rsplitn(2, '-').collect();
    if parts.len() == 2 {
        Some(parts[0].to_string())
    } else {
        None
    }
}
