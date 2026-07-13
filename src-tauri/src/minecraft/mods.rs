use serde::{Deserialize, Serialize};
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
    pub conflicts: Vec<ConflictWarning>,
    pub duplicates: Vec<DuplicateWarning>,
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

#[derive(Debug, Clone, Serialize)]
pub struct ConflictWarning {
    pub mod_a: String,
    pub mod_b: String,
    pub reason: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateWarning {
    pub category: String,
    pub installed_mods: Vec<String>,
    pub recommendation: String,
}

// --- Metadata DB structs ---

#[derive(Debug, Deserialize)]
struct ModMetadataDb {
    version: String,
    mods: Vec<ModMetadata>,
    known_conflicts: Vec<ConflictRule>,
    duplicate_groups: Vec<DuplicateGroup>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModMetadata {
    pub id: String,
    pub name: String,
    pub loaders: Vec<String>,
    pub side: String,
    pub category: String,
    pub performance_impact: Option<PerformanceImpact>,
    pub safe_removal: Option<String>,
    pub modrinth_slug: Option<String>,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PerformanceImpact {
    pub memory: String,
    pub startup: String,
    pub runtime: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ConflictRule {
    mod_a: String,
    mod_b: String,
    reason: String,
    severity: String,
}

#[derive(Debug, Deserialize, Clone)]
struct DuplicateGroup {
    category: String,
    mods: Vec<String>,
    recommendation: String,
}

fn load_metadata() -> ModMetadataDb {
    let json = include_str!("../../data/mod_metadata.json");
    serde_json::from_str(json).unwrap_or_else(|_| ModMetadataDb {
        version: "0.0.0".to_string(),
        mods: vec![],
        known_conflicts: vec![],
        duplicate_groups: vec![],
    })
}

pub fn get_metadata_version() -> String {
    load_metadata().version
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
        PerformanceModRecommendation {
            mod_name: "Distant Horizons".to_string(),
            mod_id: "distanthorizons".to_string(),
            reason: "Renders far terrain using LOD meshes, dramatically improving visual range without GPU cost. Requires client-side installation.".to_string(),
            expected_impact: "High — massively extended render distance with minimal performance cost".to_string(),
            confidence: "high".to_string(),
            url: "https://modrinth.com/mod/distanthorizons".to_string(),
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
    let (conflicts, duplicates) = detect_conflicts_and_duplicates(&mods);

    ModAnalysis {
        total_mods: mods.len(),
        mods,
        detected_performance_mods,
        missing_performance_mods,
        conflicts,
        duplicates,
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

fn detect_conflicts_and_duplicates(mods: &[ModInfo]) -> (Vec<ConflictWarning>, Vec<DuplicateWarning>) {
    let metadata = load_metadata();
    let installed_ids: Vec<String> = mods
        .iter()
        .filter_map(|m| normalized_mod_id(m))
        .collect();

    let conflicts = detect_known_conflicts(&installed_ids, &metadata.known_conflicts);
    let duplicates = detect_duplicate_groups(&installed_ids, &metadata.duplicate_groups, &metadata.mods);

    (conflicts, duplicates)
}

fn normalized_mod_id(mod_info: &ModInfo) -> Option<String> {
    let lower = mod_info.file_name.to_lowercase();
    let stem = lower
        .trim_end_matches(".jar")
        .trim_end_matches(".zip")
        .trim_end_matches(".disabled");

    if let Some(id) = &mod_info.mod_id {
        return Some(id.to_lowercase());
    }

    Some(stem.to_string())
}

fn detect_known_conflicts(installed_ids: &[String], rules: &[ConflictRule]) -> Vec<ConflictWarning> {
    let mut warnings = Vec::new();

    for rule in rules {
        let has_a = installed_ids.iter().any(|id| id.contains(&rule.mod_a));
        let has_b = installed_ids.iter().any(|id| id.contains(&rule.mod_b));

        if has_a && has_b {
            warnings.push(ConflictWarning {
                mod_a: rule.mod_a.clone(),
                mod_b: rule.mod_b.clone(),
                reason: rule.reason.clone(),
                severity: rule.severity.clone(),
            });
        }
    }

    warnings
}

fn detect_duplicate_groups(
    installed_ids: &[String],
    groups: &[DuplicateGroup],
    known_mods: &[ModMetadata],
) -> Vec<DuplicateWarning> {
    let mut warnings = Vec::new();

    for group in groups {
        let installed_in_group: Vec<String> = group
            .mods
            .iter()
            .filter(|mod_id| {
                installed_ids.iter().any(|installed| installed.contains(mod_id.as_str()))
            })
            .map(|mod_id| {
                known_mods
                    .iter()
                    .find(|m| m.id == *mod_id)
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| mod_id.clone())
            })
            .collect();

        if installed_in_group.len() > 1 {
            warnings.push(DuplicateWarning {
                category: group.category.clone(),
                installed_mods: installed_in_group,
                recommendation: group.recommendation.clone(),
            });
        }
    }

    warnings
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_loads_successfully() {
        let db = load_metadata();
        assert_eq!(db.version, "1.0.0");
        assert!(db.mods.len() >= 30);
        assert!(!db.known_conflicts.is_empty());
        assert!(!db.duplicate_groups.is_empty());
    }

    #[test]
    fn get_metadata_version_returns_version() {
        let version = get_metadata_version();
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn detects_sodium_optifine_conflict() {
        let mods = vec![
            ModInfo { file_name: "sodium-0.5.8.jar".into(), mod_id: Some("sodium".into()), display_name: None, version: Some("0.5.8".into()), size_bytes: 1000 },
            ModInfo { file_name: "OptiFine_1.20.1_HD.jar".into(), mod_id: Some("optifine".into()), display_name: None, version: None, size_bytes: 2000 },
        ];
        let (conflicts, _) = detect_conflicts_and_duplicates(&mods);
        assert!(conflicts.iter().any(|c| c.mod_a == "sodium" && c.mod_b == "optifine"));
    }

    #[test]
    fn detects_duplicate_rendering_group() {
        let mods = vec![
            ModInfo { file_name: "sodium-0.5.8.jar".into(), mod_id: Some("sodium".into()), display_name: None, version: Some("0.5.8".into()), size_bytes: 1000 },
            ModInfo { file_name: "embeddium-1.0.jar".into(), mod_id: Some("embeddium".into()), display_name: None, version: Some("1.0".into()), size_bytes: 2000 },
        ];
        let (_, duplicates) = detect_conflicts_and_duplicates(&mods);
        assert!(duplicates.iter().any(|d| d.category == "rendering_optimizer"));
    }

    #[test]
    fn no_conflicts_for_compatible_mods() {
        let mods = vec![
            ModInfo { file_name: "modernfix-5.0.jar".into(), mod_id: Some("modernfix".into()), display_name: None, version: Some("5.0".into()), size_bytes: 1000 },
            ModInfo { file_name: "ferritecore-6.0.jar".into(), mod_id: Some("ferritecore".into()), display_name: None, version: Some("6.0".into()), size_bytes: 1000 },
        ];
        let (conflicts, duplicates) = detect_conflicts_and_duplicates(&mods);
        assert!(conflicts.is_empty());
        assert!(duplicates.is_empty());
    }

    #[test]
    fn analyze_mods_returns_empty_for_nonexistent_path() {
        let analysis = analyze_mods(Path::new("/nonexistent/path"), None);
        assert_eq!(analysis.total_mods, 0);
        assert!(analysis.conflicts.is_empty());
        assert!(analysis.duplicates.is_empty());
    }
}
