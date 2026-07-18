#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod db;
pub mod events;
pub mod gamemodule;
pub mod games;
pub mod governor;
pub mod hardware;
pub mod launch;
pub mod minecraft;
pub mod overlay;
pub mod platform;
pub mod recommendations;
pub mod sessions;
pub mod sharing;
pub mod system;
pub mod telemetry;
pub mod vision;

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

#[tauri::command]
async fn analyze_disk_for_instance(
    instance_path: String,
    xmx_mb: u32,
) -> Result<system::disk_advisor::DiskAdvice, String> {
    tokio::task::spawn_blocking(move || {
        Ok(system::disk_advisor::analyze_disk_for_instance(
            &instance_path,
            xmx_mb,
        ))
    })
    .await
    .map_err(|e| e.to_string())?
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
            vision_mode: format!("{:?}", governor::current_vision_mode()),
            capture_interval_ms: governor::capture_interval_ms(),
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
async fn analyze_crashes(instance_path: String) -> Result<minecraft::crash::CrashDiagnosis, String> {
    tokio::task::spawn_blocking(move || {
        Ok(minecraft::crash::analyze_crashes(std::path::Path::new(&instance_path)))
    })
    .await
    .map_err(|e| e.to_string())?
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

// --- League of Legends ---

#[tauri::command]
async fn check_league_game_active() -> Result<bool, String> {
    Ok(games::league::is_game_active().await)
}

#[tauri::command]
async fn get_league_live_data() -> Result<serde_json::Value, String> {
    games::league::get_all_game_data().await
}

// --- Path of Exile ---

#[tauri::command]
async fn get_poe_currency_prices(
    league: String,
) -> Result<Vec<games::poe::CurrencyPrice>, String> {
    games::poe::fetch_currency_prices(&league).await
}

#[tauri::command]
async fn discover_poe_instances() -> Result<Vec<gamemodule::GameInstance>, String> {
    tokio::task::spawn_blocking(|| {
        use gamemodule::GameModule;
        let poe = games::poe::PoeModule;
        poe.discover_instances()
    })
    .await
    .map_err(|e| e.to_string())
}

// --- RuneScape ---

#[tauri::command]
async fn lookup_runescape_player(
    username: String,
    game: String,
) -> Result<games::runescape::PlayerStats, String> {
    games::runescape::lookup_player(&username, &game).await
}

#[tauri::command]
async fn lookup_ge_price(item_id: u32) -> Result<games::runescape::GrandExchangeItem, String> {
    games::runescape::lookup_ge_price(item_id).await
}

#[tauri::command]
async fn discover_runescape_instances() -> Result<Vec<gamemodule::GameInstance>, String> {
    tokio::task::spawn_blocking(|| {
        use gamemodule::GameModule;
        let osrs = games::runescape::OsrsModule;
        let rs3 = games::runescape::Rs3Module;
        let mut instances = osrs.discover_instances();
        instances.extend(rs3.discover_instances());
        instances
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
        let league = games::league::LeagueModule;
        let poe = games::poe::PoeModule;
        let tarkov = games::tarkov::TarkovModule;
        let osrs = games::runescape::OsrsModule;
        let rs3 = games::runescape::Rs3Module;
        let mut infos = vec![steam.game_info(), league.game_info(), poe.game_info(), tarkov.game_info(), osrs.game_info(), rs3.game_info()];
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

// --- Tarkov ---

#[tauri::command]
async fn get_tarkov_ammo_data() -> Result<Vec<games::tarkov::AmmoData>, String> {
    games::tarkov::fetch_ammo_data().await
}

#[tauri::command]
async fn search_tarkov_item(name: String) -> Result<Vec<games::tarkov::ItemPrice>, String> {
    games::tarkov::search_items(&name).await
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
    let (saved_java_path, saved_jvm_args) = state
        .lock()
        .ok()
        .and_then(|app| {
            let conn = app.db.conn();
            conn.query_row(
                "SELECT java_path, jvm_args FROM launch_profiles WHERE instance_id = ?1 ORDER BY updated_at DESC LIMIT 1",
                rusqlite::params![instance_id],
                |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?)),
            ).ok()
        })
        .unwrap_or((None, None));

    let profile = launch::LaunchProfile {
        instance_id: instance_id.clone(),
        launcher,
        instance_path,
        java_path: saved_java_path,
        jvm_args: saved_jvm_args,
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
fn auto_detect_and_manage_session(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<serde_json::Value>, String> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();

    let known_games: &[(&str, &str)] = &[
        ("java", "minecraft"),
        ("javaw", "minecraft"),
        ("LeagueClient", "league"),
        ("League of Legends", "league"),
        ("EscapeFromTarkov", "tarkov"),
        ("PathOfExile", "poe"),
        ("rs2client", "runescape"),
        ("runelite", "runescape"),
    ];

    let detected_game = sys.processes().values().find_map(|p| {
        let name = p.name().to_lowercase();
        known_games.iter().find_map(|(pattern, game_id)| {
            if name.contains(&pattern.to_lowercase()) {
                Some((game_id.to_string(), p.name().to_string()))
            } else {
                None
            }
        })
    });

    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();

    let active_session: Option<String> = conn
        .query_row(
            "SELECT id FROM sessions WHERE status = 'active' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    match (active_session, detected_game) {
        (None, Some((game_id, process_name))) => {
            let session = sessions::create_session(
                &app.db,
                &format!("auto-{}", game_id),
                &format!("Auto-detected: {}", process_name),
            )
            .map_err(|e| format!("Session create failed: {}", e))?;

            events::emit(
                &app.app_handle,
                &events::GamePilotEvent::GameLaunched {
                    instance_id: format!("auto-{}", game_id),
                    session_id: session.id.clone(),
                    method: "auto-detected".to_string(),
                },
            );

            Ok(Some(serde_json::json!({
                "action": "session_started",
                "session_id": session.id,
                "game": game_id,
                "process": process_name,
            })))
        }
        (Some(session_id), None) => {
            let session = sessions::end_session(&app.db, &session_id)
                .map_err(|e| format!("Session end failed: {}", e))?;

            events::emit(
                &app.app_handle,
                &events::GamePilotEvent::GameExited {
                    instance_id: session.instance_id.clone(),
                    session_id: session.id.clone(),
                },
            );

            Ok(Some(serde_json::json!({
                "action": "session_ended",
                "session_id": session.id,
            })))
        }
        (Some(session_id), Some((game_id, _))) => Ok(Some(serde_json::json!({
            "action": "active",
            "session_id": session_id,
            "game": game_id,
        }))),
        (None, None) => Ok(None),
    }
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
fn get_recommendation_statuses(
    instance_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, status FROM recommendations WHERE instance_id = ?1 AND status != 'new'"
    ).map_err(|e| format!("DB error: {}", e))?;
    let mut map = std::collections::HashMap::new();
    let rows = stmt.query_map(rusqlite::params![instance_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| format!("Query error: {}", e))?;
    for row in rows.flatten() {
        map.insert(row.0, row.1);
    }
    Ok(map)
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

    let (cfg_path, format) = if path.join("instance.cfg").exists() {
        (path.join("instance.cfg"), "prism")
    } else if path.join("minecraftinstance.json").exists() {
        (path.join("minecraftinstance.json"), "curseforge")
    } else if path.join("instance.json").exists() {
        (path.join("instance.json"), "atlauncher")
    } else if path.join("profile.json").exists() {
        (path.join("profile.json"), "modrinth")
    } else {
        return Err("No recognized launcher config found. Supported: Prism/MultiMC (instance.cfg), CurseForge (minecraftinstance.json), ATLauncher (instance.json), Modrinth (profile.json)".to_string());
    };

    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let rp = recommendations::backup_file(&cfg_path, &recommendation_id)?;
    recommendations::save_rollback_point(&app.db, &rp)?;

    let content = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let mut changes: Vec<(String, Option<String>, String)> = Vec::new();

    match format {
        "prism" => {
            let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
            let read_val = |lines: &[String], key: &str| -> Option<String> {
                let prefix = format!("{}=", key);
                lines.iter().find_map(|l| {
                    if l.starts_with(&prefix) { Some(l[prefix.len()..].to_string()) } else { None }
                })
            };
            if let Some(xmx) = xmx_mb {
                changes.push(("MaxMemAlloc".into(), read_val(&lines, "MaxMemAlloc"), xmx.to_string()));
                upsert_cfg_value(&mut lines, "MaxMemAlloc", &xmx.to_string());
            }
            if let Some(xms) = xms_mb {
                changes.push(("MinMemAlloc".into(), read_val(&lines, "MinMemAlloc"), xms.to_string()));
                upsert_cfg_value(&mut lines, "MinMemAlloc", &xms.to_string());
            }
            if let Some(ref args) = jvm_args {
                changes.push(("JvmArgs".into(), read_val(&lines, "JvmArgs"), args.clone()));
                upsert_cfg_value(&mut lines, "JvmArgs", args);
            }
            if let Some(ref java) = java_path {
                changes.push(("JavaPath".into(), read_val(&lines, "JavaPath"), java.clone()));
                upsert_cfg_value(&mut lines, "JavaPath", java);
            }
            std::fs::write(&cfg_path, lines.join("\n") + "\n")
                .map_err(|e| format!("Failed to write config: {}", e))?;
        }
        "curseforge" => {
            let mut json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse CurseForge config: {}", e))?;
            if let Some(xmx) = xmx_mb {
                let old = json.get("allocatedMemory").and_then(|v| v.as_u64()).map(|v| v.to_string());
                json["allocatedMemory"] = serde_json::json!(xmx);
                changes.push(("allocatedMemory".into(), old, xmx.to_string()));
            }
            if let Some(ref args) = jvm_args {
                let old = json.get("javaArgsOverride").and_then(|v| v.as_str()).map(String::from);
                json["javaArgsOverride"] = serde_json::json!(args);
                json["isCustomJavaArgs"] = serde_json::json!(true);
                changes.push(("javaArgsOverride".into(), old, args.clone()));
            }
            if let Some(ref java) = java_path {
                let old = json.get("javaPath").and_then(|v| v.as_str()).map(String::from);
                json["javaPath"] = serde_json::json!(java);
                json["isCustomJavaPath"] = serde_json::json!(true);
                changes.push(("javaPath".into(), old, java.clone()));
            }
            let output = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Failed to serialize config: {}", e))?;
            std::fs::write(&cfg_path, output)
                .map_err(|e| format!("Failed to write config: {}", e))?;
        }
        "atlauncher" | "modrinth" => {
            let mut json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config: {}", e))?;
            if let Some(xmx) = xmx_mb {
                let old = json.get("memory").or(json.get("maximumMemory"))
                    .and_then(|v| v.as_u64()).map(|v| v.to_string());
                json["maximumMemory"] = serde_json::json!(xmx);
                changes.push(("maximumMemory".into(), old, xmx.to_string()));
            }
            if let Some(ref args) = jvm_args {
                let old = json.get("javaArguments").or(json.get("extraArguments"))
                    .and_then(|v| v.as_str()).map(String::from);
                json["javaArguments"] = serde_json::json!(args);
                changes.push(("javaArguments".into(), old, args.clone()));
            }
            let output = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Failed to serialize config: {}", e))?;
            std::fs::write(&cfg_path, output)
                .map_err(|e| format!("Failed to write config: {}", e))?;
        }
        _ => return Err("Unsupported launcher format".to_string()),
    }

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
    fps_avg: Option<f32>,
    fps_low_1pct: Option<f32>,
    tps_avg: Option<f32>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    telemetry::store_summary(
        &app.db, &session_id, cpu_avg, ram_avg, ram_peak, hog_count,
        fps_avg, fps_low_1pct, tps_avg,
    )
}

#[tauri::command]
fn store_fps_observation(
    session_id: String,
    fps_avg: Option<f32>,
    fps_low: Option<f32>,
    tps_avg: Option<f32>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    telemetry::store_summary(
        &app.db, &session_id, 0.0, 0.0, 0.0, 0,
        fps_avg, fps_low, tps_avg,
    )
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

// --- Recommendation Outcomes ---

#[tauri::command]
fn evaluate_recommendation_outcomes(
    session_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();

    let session = sessions::get_session(&app.db, &session_id)?;

    let prev_session: Option<sessions::Session> = conn
        .query_row(
            "SELECT id, instance_id, started_at, ended_at, duration_secs, launch_method, \
             cpu_avg_percent, ram_avg_mb, ram_peak_mb, status, notes \
             FROM sessions WHERE instance_id = ?1 AND id != ?2 AND status = 'completed' \
             ORDER BY started_at DESC LIMIT 1",
            rusqlite::params![session.instance_id, session_id],
            |row| {
                Ok(sessions::Session {
                    id: row.get(0)?,
                    instance_id: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    duration_secs: row.get(4)?,
                    launch_method: row.get(5)?,
                    cpu_avg_percent: row.get(6)?,
                    ram_avg_mb: row.get(7)?,
                    ram_peak_mb: row.get(8)?,
                    status: row.get(9)?,
                    notes: row.get(10)?,
                })
            },
        )
        .ok();

    let prev = match prev_session {
        Some(p) => p,
        None => return Ok(vec![]),
    };

    let mut stmt = conn
        .prepare(
            "SELECT id, title, category FROM recommendations \
             WHERE instance_id = ?1 AND status = 'applied'",
        )
        .map_err(|e| format!("DB error: {}", e))?;

    let applied_recs: Vec<(String, String, String)> = stmt
        .query_map(rusqlite::params![session.instance_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    if applied_recs.is_empty() {
        return Ok(vec![]);
    }

    let metrics: Vec<(&str, Option<f64>, Option<f64>)> = vec![
        ("cpu_avg_percent", prev.cpu_avg_percent, session.cpu_avg_percent),
        ("ram_avg_mb", prev.ram_avg_mb, session.ram_avg_mb),
        ("ram_peak_mb", prev.ram_peak_mb, session.ram_peak_mb),
    ];

    let mut outcomes = Vec::new();

    for (rec_id, _title, _category) in &applied_recs {
        for (metric_name, before, after) in &metrics {
            let (val_before, val_after) = match (before, after) {
                (Some(b), Some(a)) => (*b, *a),
                _ => continue,
            };

            // Lower is better for all three metrics
            let improvement_pct = if val_before > 0.0 {
                ((val_before - val_after) / val_before) * 100.0
            } else {
                0.0
            };

            let outcome = if improvement_pct > 5.0 {
                "positive"
            } else if improvement_pct < -5.0 {
                "negative"
            } else {
                "neutral"
            };

            let outcome_id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT OR IGNORE INTO recommendation_outcomes \
                 (id, recommendation_id, session_before_id, session_after_id, \
                  metric_name, value_before, value_after, improvement_percent, outcome) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    outcome_id,
                    rec_id,
                    prev.id,
                    session.id,
                    metric_name,
                    val_before,
                    val_after,
                    improvement_pct,
                    outcome,
                ],
            )
            .ok();

            outcomes.push(serde_json::json!({
                "id": outcome_id,
                "recommendation_id": rec_id,
                "metric_name": metric_name,
                "value_before": val_before,
                "value_after": val_after,
                "improvement_percent": improvement_pct,
                "outcome": outcome,
            }));
        }
    }

    Ok(outcomes)
}

#[tauri::command]
fn get_recommendation_outcomes(
    instance_id: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();

    let (query, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &instance_id {
        Some(id) => (
            "SELECT ro.recommendation_id, ro.metric_name, ro.improvement_percent, ro.outcome, \
             r.title, r.category \
             FROM recommendation_outcomes ro \
             JOIN recommendations r ON r.id = ro.recommendation_id \
             WHERE r.instance_id = ?1 \
             ORDER BY ro.recorded_at DESC",
            vec![Box::new(id.clone()) as Box<dyn rusqlite::types::ToSql>],
        ),
        None => (
            "SELECT ro.recommendation_id, ro.metric_name, ro.improvement_percent, ro.outcome, \
             r.title, r.category \
             FROM recommendation_outcomes ro \
             JOIN recommendations r ON r.id = ro.recommendation_id \
             ORDER BY ro.recorded_at DESC",
            vec![],
        ),
    };

    let mut stmt = conn.prepare(query).map_err(|e| format!("DB error: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(serde_json::json!({
                "recommendation_id": row.get::<_, String>(0)?,
                "metric_name": row.get::<_, String>(1)?,
                "improvement_percent": row.get::<_, Option<f64>>(2)?,
                "outcome": row.get::<_, String>(3)?,
                "title": row.get::<_, String>(4)?,
                "category": row.get::<_, String>(5)?,
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

// --- Launch Profiles ---

#[tauri::command]
fn save_launch_profile(
    instance_id: String,
    name: String,
    java_path: Option<String>,
    jvm_args: Option<String>,
    xmx_mb: Option<i64>,
    xms_mb: Option<i64>,
    pre_launch_actions: Option<String>,
    auto_apply: Option<bool>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();

    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM launch_profiles WHERE instance_id = ?1 AND name = ?2",
            rusqlite::params![instance_id, name],
            |row| row.get(0),
        )
        .ok();

    let profile_id = existing_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let auto_apply_int: i64 = if auto_apply.unwrap_or(false) { 1 } else { 0 };

    conn.execute(
        "INSERT OR REPLACE INTO launch_profiles (id, instance_id, name, java_path, jvm_args, xmx_mb, xms_mb, pre_launch_actions, auto_apply, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))",
        rusqlite::params![profile_id, instance_id, name, java_path, jvm_args, xmx_mb, xms_mb, pre_launch_actions, auto_apply_int],
    ).map_err(|e| format!("Failed to save launch profile: {}", e))?;

    Ok(profile_id)
}

#[tauri::command]
fn get_launch_profile(
    instance_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<serde_json::Value>, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();

    let result = conn.query_row(
        "SELECT id, instance_id, name, java_path, jvm_args, xmx_mb, xms_mb, pre_launch_actions, auto_apply, created_at, updated_at \
         FROM launch_profiles WHERE instance_id = ?1 ORDER BY updated_at DESC LIMIT 1",
        rusqlite::params![instance_id],
        |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "instance_id": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "java_path": row.get::<_, Option<String>>(3)?,
                "jvm_args": row.get::<_, Option<String>>(4)?,
                "xmx_mb": row.get::<_, Option<i64>>(5)?,
                "xms_mb": row.get::<_, Option<i64>>(6)?,
                "pre_launch_actions": row.get::<_, Option<String>>(7)?,
                "auto_apply": row.get::<_, i64>(8)? != 0,
                "created_at": row.get::<_, String>(9)?,
                "updated_at": row.get::<_, String>(10)?,
            }))
        },
    );

    match result {
        Ok(profile) => Ok(Some(profile)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get launch profile: {}", e)),
    }
}

#[tauri::command]
fn delete_launch_profile(
    profile_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute(
        "DELETE FROM launch_profiles WHERE id = ?1",
        rusqlite::params![profile_id],
    ).map_err(|e| format!("Failed to delete launch profile: {}", e))?;
    Ok(())
}

// --- Process Observations ---

#[tauri::command]
fn record_process_observation(
    session_id: String,
    processes_json: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let processes: Vec<serde_json::Value> =
        serde_json::from_str(&processes_json).map_err(|e| e.to_string())?;

    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = app.db.conn();

    for p in &processes {
        let cpu = p.get("cpu_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let ram = p.get("ram_mb").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let is_hog = cpu > 15.0 || ram > 500.0;
        if !is_hog {
            continue;
        }
        conn.execute(
            "INSERT INTO process_observations (id, session_id, name, pid, cpu_percent, ram_mb, category, is_resource_hog) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                session_id,
                p.get("name").and_then(|v| v.as_str()).unwrap_or("unknown"),
                p.get("pid").and_then(|v| v.as_u64()).unwrap_or(0) as i64,
                cpu,
                ram,
                p.get("category").and_then(|v| v.as_str()).unwrap_or("unknown"),
            ],
        ).ok();
    }

    Ok(())
}

// --- Data Management ---

#[tauri::command]
fn delete_all_data(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    let conn = app.db.conn();
    conn.execute_batch(
        "DELETE FROM recommendation_outcomes; \
         DELETE FROM optimization_actions; \
         DELETE FROM telemetry_summaries; \
         DELETE FROM process_observations; \
         DELETE FROM rollback_points; \
         DELETE FROM recommendations; \
         DELETE FROM launch_profiles; \
         DELETE FROM hardware_snapshots; \
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
fn preview_config_change(
    instance_path: String,
    filename: String,
    key: String,
    new_value: String,
) -> Result<serde_json::Value, String> {
    let resolved = recommendations::resolve_config_path(
        std::path::Path::new(&instance_path),
        &filename,
    );
    let content = std::fs::read_to_string(&resolved)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    let mut before_lines = Vec::new();
    let mut after_lines = Vec::new();
    let mut old_value_line: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        before_lines.push(line.to_string());

        if let Some((k, _)) = trimmed.split_once('=').or_else(|| trimmed.split_once(':')) {
            if k.trim() == key {
                old_value_line = Some(line.to_string());
                let sep = if trimmed.contains('=') { "=" } else { ":" };
                after_lines.push(format!("{}{}{}", key, sep, new_value));
                continue;
            }
        }
        after_lines.push(line.to_string());
    }

    Ok(serde_json::json!({
        "file": filename,
        "before": before_lines.join("\n"),
        "after": after_lines.join("\n"),
        "key": key,
        "old_value": old_value_line.unwrap_or_default(),
        "new_value": new_value,
    }))
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

// --- Sharing ---

#[tauri::command]
async fn export_optimization_profile(
    instance_path: String,
    launcher: String,
) -> Result<String, String> {
    let path_clone = instance_path.clone();
    let launcher_clone = launcher.clone();
    tokio::task::spawn_blocking(move || {
        let instance = minecraft::instance::parse_instance(
            std::path::Path::new(&path_clone),
            &launcher_clone,
        );
        let hw = hardware::collect_hardware_info();
        let mod_analysis = instance
            .mods_path
            .as_ref()
            .map(|p| minecraft::mods::analyze_mods(p, instance.loader_type.as_deref()));
        let recs =
            minecraft::rules::generate_recommendations(&hw, &instance, mod_analysis.as_ref());

        let health = instance
            .mods_path
            .as_ref()
            .map(|p| {
                let analysis =
                    minecraft::mods::analyze_mods(p, instance.loader_type.as_deref());
                minecraft::health::score_modpack_health(&analysis, false)
            });

        let profile = sharing::generate_profile(
            &instance,
            &recs,
            health.map(|h| h.overall_score),
        );
        sharing::export_profile(&profile)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_optimization_profile(
    json: String,
) -> Result<sharing::OptimizationProfile, String> {
    sharing::import_profile(&json)
}

#[tauri::command]
async fn share_to_discord(
    webhook_url: String,
    instance_path: String,
    launcher: String,
) -> Result<(), String> {
    let path_clone = instance_path.clone();
    let launcher_clone = launcher.clone();
    let profile = tokio::task::spawn_blocking(move || {
        let instance = minecraft::instance::parse_instance(
            std::path::Path::new(&path_clone),
            &launcher_clone,
        );
        let hw = hardware::collect_hardware_info();
        let mod_analysis = instance
            .mods_path
            .as_ref()
            .map(|p| minecraft::mods::analyze_mods(p, instance.loader_type.as_deref()));
        let recs =
            minecraft::rules::generate_recommendations(&hw, &instance, mod_analysis.as_ref());

        let health = instance
            .mods_path
            .as_ref()
            .map(|p| {
                let analysis =
                    minecraft::mods::analyze_mods(p, instance.loader_type.as_deref());
                minecraft::health::score_modpack_health(&analysis, false)
            });

        sharing::generate_profile(&instance, &recs, health.map(|h| h.overall_score))
    })
    .await
    .map_err(|e| e.to_string())?;

    sharing::send_to_discord(&webhook_url, &profile).await
}

#[tauri::command]
async fn test_discord_webhook(webhook_url: String) -> Result<(), String> {
    sharing::send_test_to_discord(&webhook_url).await
}

// --- System: Migration & Bloatware ---

#[tauri::command]
async fn migrate_instance_to_drive(
    source_path: String,
    target_dir: String,
) -> Result<system::migration::MigrationResult, String> {
    tokio::task::spawn_blocking(move || {
        system::migration::migrate_instance(
            std::path::Path::new(&source_path),
            std::path::Path::new(&target_dir),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn delete_migrated_instance(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        system::migration::delete_old_instance(std::path::Path::new(&path))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn scan_bloatware() -> Result<system::bloatware::BloatwareReport, String> {
    tokio::task::spawn_blocking(system::bloatware::scan_bloatware)
        .await
        .map_err(|e| e.to_string())
}

// --- Vision ---

#[tauri::command]
async fn capture_screen() -> Result<vision::capture::CaptureResult, String> {
    tokio::task::spawn_blocking(|| {
        let (result, _bytes) = vision::capture::capture_screen()?;
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn detect_running_game() -> Result<Option<vision::game_detect::DetectedGame>, String> {
    tokio::task::spawn_blocking(vision::game_detect::detect_running_game)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn detect_all_running_games() -> Result<Vec<vision::game_detect::DetectedGame>, String> {
    tokio::task::spawn_blocking(vision::game_detect::detect_all_running_games)
        .await
        .map_err(|e| e.to_string())
}

// --- Overlay ---

#[tauri::command]
fn toggle_overlay(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    overlay::toggle_overlay(&app.app_handle)
}

#[tauri::command]
fn show_overlay(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    overlay::show_overlay(&app.app_handle)
}

#[tauri::command]
fn hide_overlay(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let app = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    overlay::hide_overlay(&app.app_handle)
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

            let hw = hardware::collect_hardware_info();
            let device_id = {
                use sha2::Digest;
                let mut hasher = sha2::Sha256::new();
                hasher.update(hw.hostname.as_bytes());
                hasher.update(hw.cpu_model.as_bytes());
                format!("dev-{}", hex::encode(&hasher.finalize()[..8]))
            };
            let conn = db.conn();
            conn.execute(
                "INSERT OR REPLACE INTO devices (id, hostname, os_name, os_version, cpu_model, cpu_cores, cpu_threads, gpu_model, gpu_vram_mb, ram_total_mb, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))",
                rusqlite::params![device_id, hw.hostname, hw.os_name, hw.os_version, hw.cpu_model, hw.cpu_cores, hw.cpu_threads, hw.gpu_model, hw.gpu_vram_mb, hw.ram_total_mb],
            ).ok();
            drop(conn);

            app.manage(Mutex::new(AppState { db, app_handle: handle }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_hardware_info,
            get_process_info,
            get_telemetry_sample,
            get_self_metrics,
            is_game_running,
            analyze_disk_for_instance,
            get_governor_status,
            discover_all_games,
            discover_steam_games,
            check_league_game_active,
            get_league_live_data,
            get_poe_currency_prices,
            discover_poe_instances,
            lookup_runescape_player,
            lookup_ge_price,
            discover_runescape_instances,
            discover_launchers,
            discover_all_instances,
            scan_instance,
            analyze_mods,
            get_mod_metadata_version,
            analyze_configs,
            analyze_crashes,
            get_modpack_health,
            get_recommendations,
            get_recommendations_for_path,
            update_recommendation_status,
            save_recommendation,
            evaluate_recommendation_outcomes,
            get_recommendation_outcomes,
            get_recommendation_statuses,
            detect_java,
            launch_instance,
            store_session_telemetry,
            auto_detect_and_manage_session,
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
            preview_config_change,
            apply_config_change,
            apply_config_change_auto,
            get_optimization_history,
            export_user_data,
            tail_game_log,
            store_telemetry_summary,
            store_fps_observation,
            get_telemetry_summaries,
            search_modrinth_mods,
            get_modrinth_mod_versions,
            install_modrinth_mod,
            remove_mod,
            enable_mod,
            save_launch_profile,
            get_launch_profile,
            delete_launch_profile,
            record_process_observation,
            get_tarkov_ammo_data,
            search_tarkov_item,
            export_optimization_profile,
            import_optimization_profile,
            share_to_discord,
            test_discord_webhook,
            capture_screen,
            detect_running_game,
            detect_all_running_games,
            toggle_overlay,
            show_overlay,
            hide_overlay,
            migrate_instance_to_drive,
            delete_migrated_instance,
            scan_bloatware,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
