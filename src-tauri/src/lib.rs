#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod db;
pub mod hardware;
pub mod launch;
pub mod minecraft;
pub mod platform;
pub mod recommendations;
pub mod sessions;

use db::Database;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    db: Database,
}

// --- Hardware & Process ---

#[tauri::command]
async fn get_hardware_info() -> Result<hardware::HardwareInfo, String> {
    tokio::task::spawn_blocking(hardware::collect_hardware_info)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_process_info() -> Result<Vec<hardware::ProcessInfo>, String> {
    tokio::task::spawn_blocking(hardware::collect_process_info)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_telemetry_sample() -> Result<hardware::TelemetrySample, String> {
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
    let app = state.lock().unwrap();
    let conn = app.db.conn();
    conn.execute(
        "UPDATE recommendations SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![status, recommendation_id],
    )
    .map_err(|e| format!("Failed to update recommendation: {}", e))?;
    Ok(())
}

#[tauri::command]
fn save_recommendation(
    rec_json: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let rec: minecraft::rules::Recommendation =
        serde_json::from_str(&rec_json).map_err(|e| e.to_string())?;

    let app = state.lock().unwrap();
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
        let app = state.lock().unwrap();
        match sessions::create_session(&app.db, &instance_id, &result.method) {
            Ok(session) => {
                result.session_id = Some(session.id.clone());
                log::info!("Session created: {}", session.id);
            }
            Err(e) => log::error!("Failed to create session: {}", e),
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
    let app = state.lock().unwrap();
    sessions::store_session_telemetry(&app.db, &session_id, cpu_avg, ram_avg, ram_peak)
}

#[tauri::command]
async fn get_recommendations_for_path(
    instance_path: String,
    launcher: String,
) -> Result<Vec<minecraft::rules::Recommendation>, String> {
    tokio::task::spawn_blocking(move || {
        let instance = minecraft::instance::parse_instance(
            std::path::Path::new(&instance_path),
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
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn end_session(
    session_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<sessions::Session, String> {
    let app = state.lock().unwrap();
    sessions::end_session(&app.db, &session_id)
}

#[tauri::command]
fn get_sessions(
    instance_id: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Vec<sessions::Session> {
    let app = state.lock().unwrap();
    sessions::list_sessions(&app.db, instance_id.as_deref())
}

#[tauri::command]
fn get_session_report(
    session_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<sessions::SessionReport, String> {
    let app = state.lock().unwrap();
    sessions::generate_report(&app.db, &session_id)
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
    let app = state.lock().unwrap();
    recommendations::save_rollback_point(&app.db, &rp)?;
    Ok(rp)
}

#[tauri::command]
fn rollback_file(
    rollback_point_json: String,
) -> Result<(), String> {
    let rp: recommendations::RollbackPoint =
        serde_json::from_str(&rollback_point_json).map_err(|e| e.to_string())?;
    recommendations::rollback_file(&rp)
}

// --- Instance Persistence ---

#[tauri::command]
fn delete_instance(
    instance_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().unwrap();
    let conn = app.db.conn();
    conn.execute(
        "DELETE FROM game_instances WHERE id = ?1",
        rusqlite::params![instance_id],
    )
    .map_err(|e| format!("Failed to delete instance: {}", e))?;
    Ok(())
}

#[tauri::command]
fn save_instance(
    instance_json: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let instance: minecraft::instance::MinecraftInstance =
        serde_json::from_str(&instance_json).map_err(|e| e.to_string())?;

    let app = state.lock().unwrap();
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

    Ok(())
}

#[tauri::command]
fn get_saved_instances(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Vec<serde_json::Value> {
    let app = state.lock().unwrap();
    let conn = app.db.conn();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, path, launcher, minecraft_version, loader_type, loader_version, \
             mod_count, last_played_at FROM game_instances ORDER BY updated_at DESC",
        )
        .unwrap();

    stmt.query_map([], |row| {
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
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

// --- Ignore Rules ---

#[tauri::command]
fn add_ignore_rule(
    rule_type: String,
    pattern: String,
    reason: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().unwrap();
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
) -> Vec<serde_json::Value> {
    let app = state.lock().unwrap();
    let conn = app.db.conn();
    let mut stmt = conn
        .prepare("SELECT id, rule_type, pattern, reason, created_at FROM ignore_rules ORDER BY created_at DESC")
        .unwrap();

    stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "rule_type": row.get::<_, String>(1)?,
            "pattern": row.get::<_, String>(2)?,
            "reason": row.get::<_, Option<String>>(3)?,
            "created_at": row.get::<_, String>(4)?,
        }))
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

#[tauri::command]
fn remove_ignore_rule(
    rule_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().unwrap();
    let conn = app.db.conn();
    conn.execute("DELETE FROM ignore_rules WHERE id = ?1", rusqlite::params![rule_id])
        .map_err(|e| format!("Failed to remove ignore rule: {}", e))?;
    Ok(())
}

// --- Data Management ---

#[tauri::command]
fn delete_all_data(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().unwrap();
    let conn = app.db.conn();
    conn.execute_batch(
        "DELETE FROM process_observations; \
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
    let app = state.lock().unwrap();
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
    let app = state.lock().unwrap();
    let conn = app.db.conn();
    conn.execute(
        "INSERT OR REPLACE INTO user_preferences (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
        rusqlite::params![key, value],
    )
    .map_err(|e| format!("Failed to set preference: {}", e))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("gamepilot.db");

            let db = Database::open(&db_path).expect("Failed to open database");

            app.manage(Mutex::new(AppState { db }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_hardware_info,
            get_process_info,
            get_telemetry_sample,
            get_self_metrics,
            is_game_running,
            discover_launchers,
            discover_all_instances,
            scan_instance,
            analyze_mods,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
