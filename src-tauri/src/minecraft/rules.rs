use serde::Serialize;

use crate::hardware::HardwareInfo;
use crate::minecraft::instance::MinecraftInstance;
use crate::minecraft::mods::ModAnalysis;

#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    pub id: String,
    pub category: String,
    pub severity: String,
    pub confidence: String,
    pub title: String,
    pub description: String,
    pub evidence: String,
    pub expected_impact: String,
    pub risk_level: String,
    pub action_type: Option<String>,
    pub action_data: Option<String>,
}

pub fn generate_recommendations(
    hw: &HardwareInfo,
    instance: &MinecraftInstance,
    mod_analysis: Option<&ModAnalysis>,
) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    generate_ram_recommendation(hw, instance, &mut recs);
    generate_jvm_flag_recommendations(instance, &mut recs);
    generate_java_version_recommendation(instance, &mut recs);

    if let Some(analysis) = mod_analysis {
        generate_performance_mod_recommendations(analysis, &mut recs);
        generate_mod_count_recommendation(analysis, &mut recs);
    }

    recs
}

fn generate_ram_recommendation(
    hw: &HardwareInfo,
    instance: &MinecraftInstance,
    recs: &mut Vec<Recommendation>,
) {
    let total_ram_gb = hw.ram_total_mb as f64 / 1024.0;
    let recommended_xmx_mb = recommend_xmx_mb(hw.ram_total_mb, instance.mod_count);

    let current_xmx = instance.xmx_mb;

    let title;
    let description;
    let severity;

    match current_xmx {
        Some(current) if current < recommended_xmx_mb.saturating_sub(1024) => {
            title = format!("Increase RAM allocation from {} MB to {} MB", current, recommended_xmx_mb);
            description = format!(
                "Your system has {:.0} GB RAM. With {} mods installed, \
                 allocating {} MB is recommended. Current allocation of {} MB \
                 may cause stuttering and out-of-memory crashes.",
                total_ram_gb, instance.mod_count, recommended_xmx_mb, current
            );
            severity = "warning".to_string();
        }
        Some(current) if current > recommended_xmx_mb + 2048 => {
            title = format!("Reduce RAM allocation from {} MB to {} MB", current, recommended_xmx_mb);
            description = format!(
                "Your system has {:.0} GB RAM. Allocating {} MB is excessive \
                 and may cause longer GC pauses. {} MB is recommended for {} mods.",
                total_ram_gb, current, recommended_xmx_mb, instance.mod_count
            );
            severity = "info".to_string();
        }
        Some(_) => return,
        None => {
            title = format!("Set RAM allocation to {} MB", recommended_xmx_mb);
            description = format!(
                "Your system has {:.0} GB RAM. With {} mods, \
                 {} MB is the recommended allocation for smooth gameplay.",
                total_ram_gb, instance.mod_count, recommended_xmx_mb
            );
            severity = "info".to_string();
        }
    }

    recs.push(Recommendation {
        id: uuid::Uuid::new_v4().to_string(),
        category: "java_jvm".to_string(),
        severity,
        confidence: "high".to_string(),
        title,
        description,
        evidence: format!(
            "System RAM: {} MB, Current Xmx: {} MB, Mod count: {}",
            hw.ram_total_mb,
            current_xmx.unwrap_or(0),
            instance.mod_count
        ),
        expected_impact: "Reduced stuttering, fewer out-of-memory crashes".to_string(),
        risk_level: "low".to_string(),
        action_type: Some("set_jvm_arg".to_string()),
        action_data: Some(format!("-Xmx{}m -Xms{}m", recommended_xmx_mb, recommended_xmx_mb / 2)),
    });
}

fn recommend_xmx_mb(total_ram_mb: u64, mod_count: usize) -> u32 {
    let base = match total_ram_mb {
        0..=7168 => 4096,
        7169..=12288 => 6144,
        12289..=20480 => 8192,
        20481..=36864 => 10240,
        _ => 12288,
    };

    if mod_count > 200 {
        (base as f64 * 1.3) as u32
    } else if mod_count > 100 {
        (base as f64 * 1.15) as u32
    } else {
        base
    }
}

fn generate_jvm_flag_recommendations(instance: &MinecraftInstance, recs: &mut Vec<Recommendation>) {
    let current_args = instance.jvm_args.as_deref().unwrap_or("");

    if !current_args.contains("UseG1GC") && !current_args.contains("UseShenandoahGC") && !current_args.contains("UseZGC") {
        recs.push(Recommendation {
            id: uuid::Uuid::new_v4().to_string(),
            category: "java_jvm".to_string(),
            severity: "info".to_string(),
            confidence: "high".to_string(),
            title: "Add optimized GC flags for Minecraft".to_string(),
            description: "G1GC with tuned parameters reduces GC pause times \
                          and improves frame time consistency."
                .to_string(),
            evidence: format!("Current JVM args: '{}'", current_args),
            expected_impact: "Reduced micro-stuttering from GC pauses".to_string(),
            risk_level: "low".to_string(),
            action_type: Some("set_jvm_arg".to_string()),
            action_data: Some(optimized_jvm_flags()),
        });
    }
}

fn generate_java_version_recommendation(
    instance: &MinecraftInstance,
    recs: &mut Vec<Recommendation>,
) {
    let mc_version = instance.minecraft_version.as_deref().unwrap_or("");

    let needs_java_21 = mc_version.starts_with("1.20.5")
        || mc_version.starts_with("1.21")
        || mc_version.starts_with("1.22");

    let loader = instance.loader_type.as_deref().unwrap_or("");
    let is_neoforge = loader.to_lowercase().contains("neoforge");

    if needs_java_21 || is_neoforge {
        recs.push(Recommendation {
            id: uuid::Uuid::new_v4().to_string(),
            category: "java_jvm".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            title: "Ensure Java 21 is installed".to_string(),
            description: format!(
                "Minecraft {} with {} requires Java 21. \
                 Using an older version will cause crashes or compatibility issues.",
                mc_version, loader
            ),
            evidence: format!(
                "MC version: {}, Loader: {}",
                mc_version, loader
            ),
            expected_impact: "Required for game to run correctly".to_string(),
            risk_level: "none".to_string(),
            action_type: Some("open_link".to_string()),
            action_data: Some("https://adoptium.net/temurin/releases/?version=21".to_string()),
        });
    }
}

fn generate_performance_mod_recommendations(
    analysis: &ModAnalysis,
    recs: &mut Vec<Recommendation>,
) {
    for missing in &analysis.missing_performance_mods {
        recs.push(Recommendation {
            id: uuid::Uuid::new_v4().to_string(),
            category: "modpack".to_string(),
            severity: "info".to_string(),
            confidence: missing.confidence.clone(),
            title: format!("{} is not installed", missing.mod_name),
            description: missing.reason.clone(),
            evidence: format!(
                "Scanned {} mods, {} not found in mod list",
                analysis.total_mods, missing.mod_name
            ),
            expected_impact: missing.expected_impact.clone(),
            risk_level: "low".to_string(),
            action_type: Some("open_link".to_string()),
            action_data: Some(missing.url.clone()),
        });
    }
}

fn generate_mod_count_recommendation(analysis: &ModAnalysis, recs: &mut Vec<Recommendation>) {
    if analysis.total_mods > 200 {
        recs.push(Recommendation {
            id: uuid::Uuid::new_v4().to_string(),
            category: "modpack".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            title: format!("Heavy modpack detected ({} mods)", analysis.total_mods),
            description: format!(
                "This instance has {} mods totaling {:.0} MB. \
                 Large modpacks require more RAM and may benefit from \
                 performance-focused JVM flags and performance mods.",
                analysis.total_mods, analysis.total_size_mb
            ),
            evidence: format!(
                "Mod count: {}, Total size: {:.1} MB",
                analysis.total_mods, analysis.total_size_mb
            ),
            expected_impact: "Awareness — plan RAM and settings accordingly".to_string(),
            risk_level: "none".to_string(),
            action_type: None,
            action_data: None,
        });
    }
}

pub fn optimized_jvm_flags() -> String {
    "-XX:+UseG1GC \
     -XX:+ParallelRefProcEnabled \
     -XX:MaxGCPauseMillis=200 \
     -XX:+UnlockExperimentalVMOptions \
     -XX:+DisableExplicitGC \
     -XX:G1NewSizePercent=30 \
     -XX:G1MaxNewSizePercent=40 \
     -XX:G1HeapRegionSize=8M \
     -XX:G1ReservePercent=20 \
     -XX:G1HeapWastePercent=5 \
     -XX:G1MixedGCCountTarget=4 \
     -XX:InitiatingHeapOccupancyPercent=15 \
     -XX:G1MixedGCLiveThresholdPercent=90 \
     -XX:G1RSetUpdatingPauseTimePercent=5 \
     -XX:SurvivorRatio=32 \
     -XX:+PerfDisableSharedMem \
     -XX:MaxTenuringThreshold=1"
        .to_string()
}
