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

#[tauri::command]
fn get_hardware_info() -> hardware::HardwareInfo {
    hardware::collect_hardware_info()
}

#[tauri::command]
fn get_process_info() -> Vec<hardware::ProcessInfo> {
    hardware::collect_process_info()
}

#[tauri::command]
fn discover_launchers() -> Vec<minecraft::discovery::DiscoveredLauncher> {
    minecraft::discovery::discover_launchers()
}

#[tauri::command]
fn scan_instance(path: String, launcher: String) -> minecraft::instance::MinecraftInstance {
    minecraft::instance::parse_instance(std::path::Path::new(&path), &launcher)
}

#[tauri::command]
fn analyze_mods(
    mods_path: String,
    loader: Option<String>,
) -> minecraft::mods::ModAnalysis {
    minecraft::mods::analyze_mods(
        std::path::Path::new(&mods_path),
        loader.as_deref(),
    )
}

#[tauri::command]
fn get_recommendations(
    instance_json: String,
) -> Vec<minecraft::rules::Recommendation> {
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
}

#[tauri::command]
fn detect_java() -> Vec<platform::JavaInstallation> {
    platform::detect_java_installations()
}

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

    let result = launch::launch_instance(&profile);

    if result.success {
        if let Some(ref session_id) = result.session_id {
            let app = state.lock().unwrap();
            let _ = sessions::create_session(&app.db, &instance_id, &result.method);
            log::info!("Session created: {}", session_id);
        }
    }

    result
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
            discover_launchers,
            scan_instance,
            analyze_mods,
            get_recommendations,
            detect_java,
            launch_instance,
            end_session,
            get_sessions,
            get_session_report,
            backup_file,
            rollback_file,
            save_instance,
            get_saved_instances,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
