#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod db;
pub mod events;
pub mod gamemodule;
pub mod games;
pub mod governor;
pub mod hardware;
pub mod launch;
pub mod minecraft;
pub mod platform;
pub mod recommendations;
pub mod sessions;
pub mod telemetry;

use db::Database;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    db: Database,
    app_handle: tauri::AppHandle,
}

// --- Hardware & Process ---

#[tauri::command]
async fn get_hardware_info() -> Result<hardware::HardwareInfo, String> {
    tokio::task::spawn_blocking(hardware::collect_hardware_info)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_process_info(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<hardware::ProcessInfo>, String> {
    let ignore_patterns: Vec<String> = {
        let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let conn = app.db.conn();
        let mut stmt = conn
            .prepare("SELECT pattern FROM ignore_rules WHERE rule_type = 'process'")
            .map_err(|e| format!("DB error: {}", e))?;
        let results: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        results
    };
    tokio::task::spawn_blocking(move || hardware::collect_process_info(&ignore_patterns))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_telemetry_sample() -> Result<hardware::TelemetrySample, String> {
    if governor::current_mode() == governor::GovernorMode::Paused {
        return Err("Telemetry paused by performance governor".to_string());
    }
    tokio::task::spawn_blocking(hardware::collect_telemetry_sample)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_self_metrics() -> Result<hardware::SelfMetrics, String> {
    tokio::task::spawn_blocking(hardware::collect_self_metrics)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn is_game_running(process_name: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || hardware::is_process_running(&process_name))
        .await
        .map_err(|e| e.to_string())
}

// --- Governor ---

#[tauri::command]
async fn get_governor_status() -> Result<governor::GovernorStatus, String> {
    tokio::task::spawn_blocking(|| {
        let metrics = hardware::collect_self_metrics();
        let game_running = hardware::is_process_running("java");
        let mode = governor::evaluate(metrics.cpu_percent, metrics.ram_mb, game_running);
        governor::GovernorStatus {
            mode: format!("{:?}", mode),
            self_cpu: metrics.cpu_percent,
            self_ram_mb: metrics.ram_mb,
            telemetry_interval_ms: governor::telemetry_interval_ms(),
            game_running,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

// --- Minecraft Discovery ---

#[tauri::command]
async fn discover_launchers() -> Result<Vec<minecraft::discovery::DiscoveredLauncher>, String> {
    tokio::task::spawn_blocking(minecraft::discovery::discover_launchers)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn discover_all_instances() -> Result<Vec<minecraft::discovery::DiscoveredInstance>, String> {
    tokio::task::spawn_blocking(minecraft::discovery::discover_all_instances)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_instance(path: String, launcher: String) -> Result<minecraft::instance::MinecraftInstance, String> {
    tokio::task::spawn_blocking(move || {
        minecraft::instance::parse_instance(std::path::Path::new(&path), &launcher)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn analyze_mods(
    mods_path: String,
    loader: Option<String>,
) -> Result<minecraft::mods::ModAnalysis, String> {
    tokio::task::spawn_blocking(move || {
        minecraft::mods::analyze_mods(
            std::path::Path::new(&mods_path),
            loader.as_deref(),
        )
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn analyze_configs(
    instance_path: String,
    mod_count: usize,
) -> Result<minecraft::config::ConfigAnalysis, String> {
    tokio::task::spawn_blocking(move || {
        minecraft::config::analyze_configs(
            std::path::Path::new(&instance_path),
            mod_count,
        )
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_modpack_health(
    mods_path: String,
    loader: Option<String>,
    has_config_issues: bool,
) -> Result<minecraft::health::ModpackHealth, String> {
    tokio::task::spawn_blocking(move || {
        let analysis = minecraft::mods::analyze_mods(
            std::path::Path::new(&mods_path),
            loader.as_deref(),
        );
        minecraft::health::score_modpack_health(&analysis, has_config_issues)
    })
    .await
    .map_err(|e| e.to_string())
}

// --- Game Library ---

#[tauri::command]
async fn discover_all_games() -> Result<Vec<gamemodule::GameInfo>, String> {
    tokio::task::spawn_blocking(|| {
        use gamemodule::GameModule;
        let steam = games::steam::SteamModule;
        let mut infos = vec![steam.game_info()];
        infos.insert(
            0,
            gamemodule::GameInfo {
                id: "minecraft".to_string(),
                name: "Minecraft".to_string(),
                icon: "\u{25A3}".to_string(),
                installed: true,
                install_path: None,
            },
        );
        infos
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn discover_steam_games() -> Result<Vec<gamemodule::GameInstance>, String> {
    tokio::task::spawn_blocking(|| {
        use gamemodule::GameModule;
        let steam = games::steam::SteamModule;
        steam.discover_instances()
    })
    .await
    .map_err(|e| e.to_string())
}

// --- Recommendations ---

#[tauri::command]
async fn get_recommendations(
    instance_json: String,
) -> Result<Vec<minecraft::rules::Recommendation>, String> {
    tokio::task::spawn_blocking(move || {
        let instance: minecraft::instance::MinecraftInstance =
            serde_json::from_str(&instance_json).unwrap_or_else(|_| {
                minecraft::instance::parse_instance(std::path::Path::new(""), "unknown")
            });

        let hw = hardware::collect_hardware_info();

        let mod_analysis = instance
            .mods_path
            .as_ref()
            .map(|p| minecraft::mods::analyze_mods(p, instance.loader_type.as_deref()));

        minecraft::rules::generate_recommendations(&hw, &instance, mod_analysis.as_ref())
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_recommendation_status(
    recommendation_id: String,
    status: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let valid_statuses = ["new", "accepted", "applied", "ignored_once", "ignored_always", "deferred", "rolled_back", "failed"];
    if !valid_statuses.contains(&status.as_str()) {
        return Err(format!("Invalid status: {}", status));
    }
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();

    let old_status: String = conn
        .query_row(
            "SELECT status FROM recommendations WHERE id = ?1",
            rusqlite::params![recommendation_id],
            |row| row.get(0),
        )
        .map_err(|_| "Recommendation not found".to_string())?;

    let rows = conn.execute(
        "UPDATE recommendations SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![status, recommendation_id],
    )
    .map_err(|e| format!("Failed to update recommendation: {}", e))?;
    if rows == 0 {
        return Err("Recommendation not found".to_string());
    }

    events::emit(&app.app_handle, &events::GamePilotEvent::RecommendationStatusChanged {
        recommendation_id,
        old_status,
        new_status: status,
    });

    Ok(())
}

#[tauri::command]
fn save_recommendation(
    rec_json: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let rec: minecraft::rules::Recommendation =
        serde_json::from_str(&rec_json).map_err(|e| e.to_string())?;

    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute(
        "INSERT OR REPLACE INTO recommendations \
         (id, category, severity, confidence, title, description, evidence, \
          expected_impact, risk_level, action_type, action_data, status) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'new')",
        rusqlite::params![
            rec.id, rec.category, rec.severity, rec.confidence,
            rec.title, rec.description, rec.evidence, rec.expected_impact,
            rec.risk_level, rec.action_type, rec.action_data,
        ],
    )
    .map_err(|e| format!("Failed to save recommendation: {}", e))?;
    Ok(())
}

// --- Java ---

#[tauri::command]
async fn detect_java() -> Result<Vec<platform::JavaInstallation>, String> {
    tokio::task::spawn_blocking(platform::detect_java_installations)
        .await
        .map_err(|e| e.to_string())
}

// --- Launch & Sessions ---

#[tauri::command]
fn launch_instance(
    instance_id: String,
    launcher: String,
    instance_path: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> launch::LaunchResult {
    let profile = launch::LaunchProfile {
        instance_id: instance_id.clone(),
        launcher,
        instance_path,
        java_path: None,
        jvm_args: None,
    };

    let mut result = launch::launch_instance(&profile);

    if result.success {
        if let Ok(app) = state.lock() {
            match sessions::create_session(&app.db, &instance_id, &result.method) {
                Ok(session) => {
                    result.session_id = Some(session.id.clone());
                    log::info!("Session created: {}", session.id);
                    events::emit(&app.app_handle, &events::GamePilotEvent::GameLaunched {
                        instance_id,
                        session_id: session.id,
                        method: result.method.clone(),
                    });
                }
                Err(e) => log::error!("Failed to create session: {}", e),
            }
        } else {
            log::error!("Failed to acquire state lock for session creation");
        }
    }

    result
}

#[tauri::command]
fn store_session_telemetry(
    session_id: String,
    cpu_avg: f64,
    ram_avg: f64,
    ram_peak: f64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    sessions::store_session_telemetry(&app.db, &session_id, cpu_avg, ram_avg, ram_peak)
}

#[tauri::command]
async fn get_recommendations_for_path(
    instance_path: String,
    launcher: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<minecraft::rules::Recommendation>, String> {
    let path_clone = instance_path.clone();
    let recs = tokio::task::spawn_blocking(move || {
        let instance = minecraft::instance::parse_instance(
            std::path::Path::new(&path_clone),
            &launcher,
        );
        let hw = hardware::collect_hardware_info();
        let mod_analysis = instance
            .mods_path
            .as_ref()
            .map(|p| minecraft::mods::analyze_mods(p, instance.loader_type.as_deref()));
        minecraft::rules::generate_recommendations(&hw, &instance, mod_analysis.as_ref())
    })
    .await
    .map_err(|e| e.to_string())?;

    let instance_id = compute_instance_id(&instance_path);

    if let Ok(app) = state.lock() {
        let conn = app.db.conn();
        for rec in &recs {
            conn.execute(
                "INSERT OR IGNORE INTO recommendations \
                 (id, instance_id, category, severity, confidence, title, description, evidence, \
                  expected_impact, risk_level, action_type, action_data, status) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 'new')",
                rusqlite::params![
                    rec.id, instance_id, rec.category, rec.severity, rec.confidence,
                    rec.title, rec.description, rec.evidence, rec.expected_impact,
                    rec.risk_level, rec.action_type, rec.action_data,
                ],
            ).ok();
        }
    }

    Ok(recs)
}

#[tauri::command]
fn end_session(
    session_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<sessions::Session, String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    sessions::end_session(&app.db, &session_id)
}

#[tauri::command]
fn get_sessions(
    instance_id: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<sessions::Session>, String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    sessions::list_sessions(&app.db, instance_id.as_deref())
}

#[tauri::command]
fn get_session_report(
    session_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<sessions::SessionReport, String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    sessions::generate_report(&app.db, &session_id)
}

// --- Mod Metadata ---

#[tauri::command]
fn get_mod_metadata_version() -> String {
    minecraft::mods::get_metadata_version()
}

// --- Mod Store (Modrinth) ---

#[tauri::command]
async fn search_modrinth_mods(
    query: String,
    mc_version: Option<String>,
    loader: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<minecraft::modstore::ModSearchResult>, String> {
    minecraft::modstore::search_mods(
        &query,
        mc_version.as_deref(),
        loader.as_deref(),
        limit.unwrap_or(20),
    )
    .await
}

#[tauri::command]
async fn get_modrinth_mod_versions(
    project_id: String,
    mc_version: Option<String>,
    loader: Option<String>,
) -> Result<Vec<minecraft::modstore::ModVersion>, String> {
    minecraft::modstore::get_mod_versions(
        &project_id,
        mc_version.as_deref(),
        loader.as_deref(),
    )
    .await
}

#[tauri::command]
async fn install_modrinth_mod(
    download_url: String,
    filename: String,
    mods_dir: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<minecraft::modstore::InstallResult, String> {
    let app_handle = {
        let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app.app_handle.clone()
    };

    let result = minecraft::modstore::install_mod(
        &download_url,
        &filename,
        std::path::Path::new(&mods_dir),
    )
    .await?;

    if result.success {
        let instance_path = std::path::Path::new(&mods_dir)
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .to_string_lossy();
        let instance_id = compute_instance_id(&instance_path);
        let mod_name = std::path::Path::new(&filename)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        events::emit(&app_handle, &events::GamePilotEvent::ModInstalled {
            instance_id,
            mod_name,
            filename: filename.clone(),
        });
    }

    Ok(result)
}

#[tauri::command]
async fn remove_mod(mods_dir: String, filename: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        minecraft::modstore::remove_mod(std::path::Path::new(&mods_dir), &filename)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn enable_mod(mods_dir: String, filename: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        minecraft::modstore::enable_mod(std::path::Path::new(&mods_dir), &filename)
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- JVM Settings ---

#[tauri::command]
fn apply_jvm_settings(
    instance_path: String,
    xmx_mb: Option<u32>,
    xms_mb: Option<u32>,
    jvm_args: Option<String>,
    java_path: Option<String>,
    recommendation_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<recommendations::RollbackPoint, String> {
    let path = std::path::Path::new(&instance_path);
    let cfg_path = path.join("instance.cfg");
    if !cfg_path.exists() {
        return Err("instance.cfg not found — JVM settings apply is supported for Prism/MultiMC instances".to_string());
    }

    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let rp = recommendations::backup_file(&cfg_path, &recommendation_id)?;
    recommendations::save_rollback_point(&app.db, &rp)?;

    let content = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let read_cfg_value = |lines: &[String], key: &str| -> Option<String> {
        let prefix = format!("{}=", key);
        lines.iter().find_map(|l| {
            if l.starts_with(&prefix) { Some(l[prefix.len()..].to_string()) } else { None }
        })
    };

    let mut changes: Vec<(String, Option<String>, String)> = Vec::new();

    if let Some(xmx) = xmx_mb {
        let old = read_cfg_value(&lines, "MaxMemAlloc");
        upsert_cfg_value(&mut lines, "MaxMemAlloc", &xmx.to_string());
        changes.push(("MaxMemAlloc".to_string(), old, xmx.to_string()));
    }
    if let Some(xms) = xms_mb {
        let old = read_cfg_value(&lines, "MinMemAlloc");
        upsert_cfg_value(&mut lines, "MinMemAlloc", &xms.to_string());
        changes.push(("MinMemAlloc".to_string(), old, xms.to_string()));
    }
    if let Some(ref args) = jvm_args {
        let old = read_cfg_value(&lines, "JvmArgs");
        upsert_cfg_value(&mut lines, "JvmArgs", args);
        changes.push(("JvmArgs".to_string(), old, args.clone()));
    }
    if let Some(ref java) = java_path {
        let old = read_cfg_value(&lines, "JavaPath");
        upsert_cfg_value(&mut lines, "JavaPath", java);
        changes.push(("JavaPath".to_string(), old, java.clone()));
    }

    let new_content = lines.join("\n") + "\n";
    std::fs::write(&cfg_path, &new_content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    let instance_id = compute_instance_id(&instance_path);
    let conn = app.db.conn();
    for (key, old, new_val) in &changes {
        conn.execute(
            "INSERT INTO optimization_actions (id, recommendation_id, instance_id, action_type, description, file_path, old_value, new_value) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                recommendation_id,
                instance_id,
                "jvm_settings",
                format!("Set {} to {}", key, new_val),
                cfg_path.to_string_lossy().to_string(),
                old.as_deref(),
                new_val,
            ],
        ).ok();
    }

    Ok(rp)
}

fn compute_instance_id(instance_path: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(instance_path.as_bytes());
    let hash = hasher.finalize();
    format!("inst-{}", hex::encode(&hash[..12]))
}

fn upsert_cfg_value(lines: &mut Vec<String>, key: &str, value: &str) {
    let prefix = format!("{}=", key);
    if let Some(line) = lines.iter_mut().find(|l| l.starts_with(&prefix)) {
        *line = format!("{}{}", prefix, value);
    } else {
        lines.push(format!("{}{}", prefix, value));
    }
}

// --- Backup / Rollback ---

#[tauri::command]
fn backup_file(
    file_path: String,
    recommendation_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<recommendations::RollbackPoint, String> {
    let rp = recommendations::backup_file(
        std::path::Path::new(&file_path),
        &recommendation_id,
    )?;
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    recommendations::save_rollback_point(&app.db, &rp)?;
    Ok(rp)
}

#[tauri::command]
fn rollback_file(
    rollback_point_json: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let rp: recommendations::RollbackPoint =
        serde_json::from_str(&rollback_point_json).map_err(|e| e.to_string())?;
    recommendations::rollback_file(&rp)?;

    if let Ok(app) = state.lock() {
        let conn = app.db.conn();
        conn.execute(
            "UPDATE optimization_actions SET status = 'rolled_back', rolled_back_at = datetime('now') \
             WHERE file_path = ?1 AND status = 'applied'",
            rusqlite::params![rp.file_path],
        ).ok();

        events::emit(&app.app_handle, &events::GamePilotEvent::OptimizationRolledBack {
            recommendation_id: rp.recommendation_id.clone(),
            file_path: rp.file_path.clone(),
        });
    }
    Ok(())
}

// --- Instance Persistence ---

#[tauri::command]
fn delete_instance(
    instance_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute(
        "DELETE FROM game_instances WHERE id = ?1",
        rusqlite::params![instance_id],
    )
    .map_err(|e| format!("Failed to delete instance: {}", e))?;

    events::emit(&app.app_handle, &events::GamePilotEvent::InstanceRemoved {
        instance_id,
    });

    Ok(())
}

#[tauri::command]
fn save_instance(
    instance_json: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let instance: minecraft::instance::MinecraftInstance =
        serde_json::from_str(&instance_json).map_err(|e| e.to_string())?;

    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute(
        "INSERT OR REPLACE INTO game_instances \
         (id, game_type, name, path, launcher, minecraft_version, loader_type, loader_version, \
          java_path, jvm_args, xmx_mb, xms_mb, mod_count, updated_at) \
         VALUES (?1, 'minecraft', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))",
        rusqlite::params![
            instance.id,
            instance.name,
            instance.path.to_string_lossy(),
            instance.launcher,
            instance.minecraft_version,
            instance.loader_type,
            instance.loader_version,
            instance.java_path,
            instance.jvm_args,
            instance.xmx_mb,
            instance.xms_mb,
            instance.mod_count as i64,
        ],
    )
    .map_err(|e| format!("Failed to save instance: {}", e))?;

    events::emit(&app.app_handle, &events::GamePilotEvent::InstanceAdded {
        instance_id: instance.id,
        name: instance.name,
    });

    Ok(())
}

#[tauri::command]
fn get_saved_instances(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, path, launcher, minecraft_version, loader_type, loader_version, \
             mod_count, last_played_at FROM game_instances ORDER BY updated_at DESC",
        )
        .map_err(|e| format!("DB error: {}", e))?;

    let instances = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "name": row.get::<_, String>(1)?,
            "path": row.get::<_, String>(2)?,
            "launcher": row.get::<_, Option<String>>(3)?,
            "minecraft_version": row.get::<_, Option<String>>(4)?,
            "loader_type": row.get::<_, Option<String>>(5)?,
            "loader_version": row.get::<_, Option<String>>(6)?,
            "mod_count": row.get::<_, Option<i64>>(7)?,
            "last_played_at": row.get::<_, Option<String>>(8)?,
        }))
    })
    .map_err(|e| format!("DB query error: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(instances)
}

// --- Ignore Rules ---

#[tauri::command]
fn add_ignore_rule(
    rule_type: String,
    pattern: String,
    reason: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute(
        "INSERT INTO ignore_rules (id, rule_type, pattern, reason) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![uuid::Uuid::new_v4().to_string(), rule_type, pattern, reason],
    )
    .map_err(|e| format!("Failed to add ignore rule: {}", e))?;
    Ok(())
}

#[tauri::command]
fn get_ignore_rules(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    let mut stmt = conn
        .prepare("SELECT id, rule_type, pattern, reason, created_at FROM ignore_rules ORDER BY created_at DESC")
        .map_err(|e| format!("DB error: {}", e))?;

    let rules = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "rule_type": row.get::<_, String>(1)?,
            "pattern": row.get::<_, String>(2)?,
            "reason": row.get::<_, Option<String>>(3)?,
            "created_at": row.get::<_, String>(4)?,
        }))
    })
    .map_err(|e| format!("DB query error: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(rules)
}

#[tauri::command]
fn remove_ignore_rule(
    rule_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute("DELETE FROM ignore_rules WHERE id = ?1", rusqlite::params![rule_id])
        .map_err(|e| format!("Failed to remove ignore rule: {}", e))?;
    Ok(())
}

// --- Telemetry ---

#[tauri::command]
async fn tail_game_log(
    instance_path: String,
    from_pos: u64,
) -> Result<(Vec<telemetry::LogEvent>, u64), String> {
    tokio::task::spawn_blocking(move || {
        let log_path = telemetry::find_log_path(std::path::Path::new(&instance_path));
        match log_path {
            Some(p) => Ok(telemetry::tail_minecraft_log(&p, from_pos)),
            None => Ok((Vec::new(), 0)),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn store_telemetry_summary(
    session_id: String,
    cpu_avg: f64,
    ram_avg: f64,
    ram_peak: f64,
    hog_count: i32,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    telemetry::store_summary(&app.db, &session_id, cpu_avg, ram_avg, ram_peak, hog_count)
}

#[tauri::command]
fn get_telemetry_summaries(
    session_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<telemetry::TelemetrySummary>, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    telemetry::get_summaries(&app.db, &session_id)
}

// --- Optimization History ---

#[tauri::command]
fn get_optimization_history(
    instance_id: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();
    let (query, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &instance_id {
        Some(id) => (
            "SELECT id, recommendation_id, instance_id, action_type, description, file_path, old_value, new_value, status, applied_at, rolled_back_at \
             FROM optimization_actions WHERE instance_id = ?1 ORDER BY applied_at DESC LIMIT 100",
            vec![Box::new(id.clone()) as Box<dyn rusqlite::types::ToSql>],
        ),
        None => (
            "SELECT id, recommendation_id, instance_id, action_type, description, file_path, old_value, new_value, status, applied_at, rolled_back_at \
             FROM optimization_actions ORDER BY applied_at DESC LIMIT 100",
            vec![],
        ),
    };

    let mut stmt = conn.prepare(query).map_err(|e| format!("DB error: {}", e))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "recommendation_id": row.get::<_, Option<String>>(1)?,
            "instance_id": row.get::<_, Option<String>>(2)?,
            "action_type": row.get::<_, String>(3)?,
            "description": row.get::<_, String>(4)?,
            "file_path": row.get::<_, Option<String>>(5)?,
            "old_value": row.get::<_, Option<String>>(6)?,
            "new_value": row.get::<_, Option<String>>(7)?,
            "status": row.get::<_, String>(8)?,
            "applied_at": row.get::<_, String>(9)?,
            "rolled_back_at": row.get::<_, Option<String>>(10)?,
        }))
    })
    .map_err(|e| format!("Query error: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(rows)
}

// --- Data Export ---

#[tauri::command]
fn export_user_data(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();

    let query_json_array = |sql: &str| -> Vec<serde_json::Value> {
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let col_count = stmt.column_count();
        let col_names: Vec<String> = (0..col_count)
            .map(|i| stmt.column_name(i).unwrap_or("").to_string())
            .collect();
        let result = stmt.query_map([], |row| {
            let mut map = serde_json::Map::new();
            for (i, name) in col_names.iter().enumerate() {
                let val: rusqlite::Result<Option<String>> = row.get(i);
                map.insert(
                    name.clone(),
                    match val {
                        Ok(Some(s)) => serde_json::Value::String(s),
                        _ => serde_json::Value::Null,
                    },
                );
            }
            Ok(serde_json::Value::Object(map))
        });
        match result {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(_) => vec![],
        }
    };

    Ok(serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "version": "0.5.0",
        "instances": query_json_array("SELECT * FROM game_instances"),
        "sessions": query_json_array("SELECT * FROM sessions"),
        "recommendations": query_json_array("SELECT * FROM recommendations"),
        "optimization_actions": query_json_array("SELECT * FROM optimization_actions"),
        "preferences": query_json_array("SELECT * FROM user_preferences"),
    }))
}

// --- Data Management ---

#[tauri::command]
fn delete_all_data(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute_batch(
        "DELETE FROM optimization_actions; \
         DELETE FROM telemetry_summaries; \
         DELETE FROM process_observations; \
         DELETE FROM rollback_points; \
         DELETE FROM recommendations; \
         DELETE FROM sessions; \
         DELETE FROM game_instances; \
         DELETE FROM ignore_rules; \
         DELETE FROM user_preferences;"
    )
    .map_err(|e| format!("Failed to delete data: {}", e))?;
    Ok(())
}

#[tauri::command]
fn get_preference(
    key: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Option<String> {
    let app = match state.lock() {
        Ok(a) => a,
        Err(_) => return None,
    };
    let conn = app.db.conn();
    conn.query_row(
        "SELECT value FROM user_preferences WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    )
    .ok()
}

#[tauri::command]
fn set_preference(
    key: String,
    value: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute(
        "INSERT OR REPLACE INTO user_preferences (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
        rusqlite::params![key, value],
    )
    .map_err(|e| format!("Failed to set preference: {}", e))?;
    Ok(())
}

#[tauri::command]
fn apply_config_change(
    file_path: String,
    key: String,
    new_value: String,
    recommendation_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<recommendations::RollbackPoint, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let rp = recommendations::apply_config_change(
        std::path::Path::new(&file_path),
        &key,
        &new_value,
        &recommendation_id,
        &app.db,
    )?;

    events::emit(&app.app_handle, &events::GamePilotEvent::OptimizationApplied {
        recommendation_id,
        file_path,
    });

    Ok(rp)
}

#[tauri::command]
fn apply_config_change_auto(
    instance_path: String,
    filename: String,
    key: String,
    new_value: String,
    recommendation_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<recommendations::RollbackPoint, String> {
    let resolved = recommendations::resolve_config_path(
        std::path::Path::new(&instance_path),
        &filename,
    );

    let old_value = std::fs::read_to_string(&resolved)
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=').or_else(|| trimmed.split_once(':')) {
                    if k.trim() == key { Some(v.trim().to_string()) } else { None }
                } else {
                    None
                }
            })
        });

    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let rp = recommendations::apply_config_change(&resolved, &key, &new_value, &recommendation_id, &app.db)?;

    let instance_id = compute_instance_id(&instance_path);
    let conn = app.db.conn();
    conn.execute(
        "INSERT INTO optimization_actions (id, recommendation_id, instance_id, action_type, description, file_path, old_value, new_value) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            recommendation_id,
            instance_id,
            "config_change",
            format!("Changed {} from {} to {}", key, old_value.as_deref().unwrap_or("(unset)"), new_value),
            resolved.to_string_lossy().to_string(),
            old_value,
            new_value,
        ],
    ).ok();

    let file_path_str = resolved.to_string_lossy().to_string();
    events::emit(&app.app_handle, &events::GamePilotEvent::OptimizationApplied {
        recommendation_id,
        file_path: file_path_str,
    });

    Ok(rp)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("gamepilot.db");

            let db = Database::open(&db_path).expect("Failed to open database");
            let handle = app.handle().clone();

            app.manage(Mutex::new(AppState { db, app_handle: handle }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_hardware_info,
            get_process_info,
            get_telemetry_sample,
            get_self_metrics,
            is_game_running,
            get_governor_status,
            discover_all_games,
            discover_steam_games,
            discover_launchers,
            discover_all_instances,
            scan_instance,
            analyze_mods,
            get_mod_metadata_version,
            analyze_configs,
            get_modpack_health,
            get_recommendations,
            get_recommendations_for_path,
            update_recommendation_status,
            save_recommendation,
            detect_java,
            launch_instance,
            store_session_telemetry,
            end_session,
            get_sessions,
            get_session_report,
            apply_jvm_settings,
            backup_file,
            rollback_file,
            delete_instance,
            save_instance,
            get_saved_instances,
            add_ignore_rule,
            get_ignore_rules,
            remove_ignore_rule,
            delete_all_data,
            get_preference,
            set_preference,
            apply_config_change,
            apply_config_change_auto,
            get_optimization_history,
            export_user_data,
            tail_game_log,
            store_telemetry_summary,
            get_telemetry_summaries,
            search_modrinth_mods,
            get_modrinth_mod_versions,
            install_modrinth_mod,
            remove_mod,
            enable_mod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
