use serde::Serialize;

use crate::minecraft::mods::ModAnalysis;

#[derive(Debug, Clone, Serialize)]
pub struct ModpackHealth {
    pub overall_score: u32,
    pub memory_risk: RiskScore,
    pub rendering_risk: RiskScore,
    pub startup_risk: RiskScore,
    pub dependency_risk: RiskScore,
    pub optimization_score: RiskScore,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskScore {
    pub score: u32,
    pub label: String,
    pub detail: String,
}

pub fn score_modpack_health(analysis: &ModAnalysis, has_config_issues: bool) -> ModpackHealth {
    let memory_risk = score_memory_risk(analysis);
    let rendering_risk = score_rendering_risk(analysis);
    let startup_risk = score_startup_risk(analysis);
    let dependency_risk = score_dependency_risk(analysis);
    let optimization_score = score_optimization(analysis);

    let mut total = (memory_risk.score + rendering_risk.score + startup_risk.score
        + dependency_risk.score + optimization_score.score) / 5;

    if has_config_issues {
        total = total.saturating_sub(10);
    }

    let summary = match total {
        80..=100 => "This modpack is well-optimized with low risk.".to_string(),
        60..=79 => "This modpack is in reasonable shape but has room for improvement.".to_string(),
        40..=59 => "This modpack has several areas that could be improved for better performance.".to_string(),
        _ => "This modpack has significant performance risks that should be addressed.".to_string(),
    };

    ModpackHealth {
        overall_score: total,
        memory_risk,
        rendering_risk,
        startup_risk,
        dependency_risk,
        optimization_score,
        summary,
    }
}

fn score_memory_risk(analysis: &ModAnalysis) -> RiskScore {
    let score = match analysis.total_mods {
        0..=50 => 90,
        51..=100 => 75,
        101..=200 => 55,
        201..=300 => 35,
        _ => 20,
    };

    let size_penalty = if analysis.total_size_mb > 500.0 {
        15
    } else if analysis.total_size_mb > 200.0 {
        5
    } else {
        0
    };

    let has_ferrite = analysis.detected_performance_mods.iter()
        .any(|m| m.to_lowercase().contains("ferritecore"));
    let has_modernfix = analysis.detected_performance_mods.iter()
        .any(|m| m.to_lowercase().contains("modernfix"));

    let bonus = if has_ferrite { 10 } else { 0 } + if has_modernfix { 5 } else { 0 };
    let final_score = (score - size_penalty + bonus).clamp(0, 100);

    let label = risk_label(final_score);
    RiskScore {
        score: final_score as u32,
        label,
        detail: format!(
            "{} mods, {:.0} MB total{}{}",
            analysis.total_mods,
            analysis.total_size_mb,
            if has_ferrite { ", FerriteCore installed" } else { "" },
            if has_modernfix { ", ModernFix installed" } else { "" },
        ),
    }
}

fn score_rendering_risk(analysis: &ModAnalysis) -> RiskScore {
    let has_sodium = analysis.detected_performance_mods.iter()
        .any(|m| {
            let l = m.to_lowercase();
            l.contains("sodium") || l.contains("embeddium")
        });
    let has_entity_culling = analysis.detected_performance_mods.iter()
        .any(|m| m.to_lowercase().contains("entity culling"));

    let base = if analysis.total_mods > 150 { 50 } else { 70 };
    let bonus = if has_sodium { 20 } else { 0 } + if has_entity_culling { 10 } else { 0 };
    let score = (base + bonus).min(100);

    RiskScore {
        score: score as u32,
        label: risk_label(score),
        detail: format!(
            "Rendering optimizer: {}{}",
            if has_sodium { "present" } else { "missing" },
            if has_entity_culling { ", entity culling present" } else { "" },
        ),
    }
}

fn score_startup_risk(analysis: &ModAnalysis) -> RiskScore {
    let score = match analysis.total_mods {
        0..=30 => 95,
        31..=80 => 80,
        81..=150 => 65,
        151..=250 => 45,
        _ => 25,
    };

    RiskScore {
        score: score as u32,
        label: risk_label(score),
        detail: format!(
            "{} mods will affect startup time",
            analysis.total_mods
        ),
    }
}

fn score_dependency_risk(analysis: &ModAnalysis) -> RiskScore {
    // Without deep JAR introspection, approximate from mod count
    let score = if analysis.total_mods > 250 {
        40
    } else if analysis.total_mods > 150 {
        60
    } else {
        80
    };

    RiskScore {
        score: score as u32,
        label: risk_label(score as i32),
        detail: format!(
            "Dependency complexity scales with {} mods",
            analysis.total_mods
        ),
    }
}

fn score_optimization(analysis: &ModAnalysis) -> RiskScore {
    let perf_mod_count = analysis.detected_performance_mods.len();
    let missing_count = analysis.missing_performance_mods.len();

    let score = if missing_count == 0 {
        95
    } else {
        let base = 90i32 - (missing_count as i32 * 12);
        let bonus = perf_mod_count as i32 * 8;
        (base + bonus).clamp(10, 95) as u32
    };

    RiskScore {
        score,
        label: risk_label(score as i32),
        detail: format!(
            "{} performance mods installed, {} recommended mods missing",
            perf_mod_count, missing_count
        ),
    }
}

fn risk_label(score: i32) -> String {
    match score {
        80..=100 => "Good".to_string(),
        60..=79 => "Fair".to_string(),
        40..=59 => "Needs Attention".to_string(),
        _ => "High Risk".to_string(),
    }
}
