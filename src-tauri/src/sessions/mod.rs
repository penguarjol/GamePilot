use crate::db::Database;
use crate::hardware;
use crate::telemetry;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: String,
    pub instance_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_secs: Option<i64>,
    pub launch_method: Option<String>,
    pub cpu_avg_percent: Option<f64>,
    pub ram_avg_mb: Option<f64>,
    pub ram_peak_mb: Option<f64>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionReport {
    pub session: Session,
    pub recommendations_applied: usize,
    pub process_observations: usize,
    pub avg_fps: Option<f32>,
    pub low_1pct_fps: Option<f32>,
    pub avg_tps: Option<f32>,
    pub telemetry_points: usize,
    pub summary: String,
}

pub fn create_session(db: &Database, instance_id: &str, launch_method: &str) -> Result<Session, String> {
    let session = Session {
        id: uuid::Uuid::new_v4().to_string(),
        instance_id: instance_id.to_string(),
        started_at: chrono::Utc::now().to_rfc3339(),
        ended_at: None,
        duration_secs: None,
        launch_method: Some(launch_method.to_string()),
        cpu_avg_percent: None,
        ram_avg_mb: None,
        ram_peak_mb: None,
        status: "active".to_string(),
        notes: None,
    };

    let conn = db.conn();
    conn.execute(
        "INSERT INTO sessions (id, instance_id, started_at, launch_method, status) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![session.id, session.instance_id, session.started_at, session.launch_method, session.status],
    ).map_err(|e| format!("Failed to create session: {}", e))?;

    store_hardware_snapshot(&conn, &session.id);

    Ok(session)
}

pub fn end_session(db: &Database, session_id: &str) -> Result<Session, String> {
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    let conn = db.conn();

    let started_at: String = conn
        .query_row(
            "SELECT started_at FROM sessions WHERE id = ?1",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Session not found: {}", e))?;

    let duration_secs = chrono::DateTime::parse_from_rfc3339(&started_at)
        .ok()
        .map(|start| (now - start.with_timezone(&chrono::Utc)).num_seconds())
        .filter(|&d| d >= 0);

    conn.execute(
        "UPDATE sessions SET ended_at = ?1, duration_secs = ?2, status = 'completed' WHERE id = ?3",
        rusqlite::params![now_str, duration_secs, session_id],
    )
    .map_err(|e| format!("Failed to end session: {}", e))?;

    get_session(db, session_id)
}

pub fn store_session_telemetry(
    db: &Database,
    session_id: &str,
    cpu_avg: f64,
    ram_avg: f64,
    ram_peak: f64,
) -> Result<(), String> {
    let conn = db.conn();
    conn.execute(
        "UPDATE sessions SET cpu_avg_percent = ?1, ram_avg_mb = ?2, ram_peak_mb = ?3 WHERE id = ?4",
        rusqlite::params![cpu_avg, ram_avg, ram_peak, session_id],
    )
    .map_err(|e| format!("Failed to store telemetry: {}", e))?;
    Ok(())
}

pub fn get_session(db: &Database, session_id: &str) -> Result<Session, String> {
    let conn = db.conn();
    conn.query_row(
        "SELECT id, instance_id, started_at, ended_at, duration_secs, launch_method, cpu_avg_percent, ram_avg_mb, ram_peak_mb, status, notes FROM sessions WHERE id = ?1",
        rusqlite::params![session_id],
        |row| {
            Ok(Session {
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
    ).map_err(|e| format!("Session not found: {}", e))
}

pub fn list_sessions(db: &Database, instance_id: Option<&str>) -> Result<Vec<Session>, String> {
    let conn = db.conn();

    let query = match instance_id {
        Some(_) => "SELECT id, instance_id, started_at, ended_at, duration_secs, launch_method, cpu_avg_percent, ram_avg_mb, ram_peak_mb, status, notes FROM sessions WHERE instance_id = ?1 ORDER BY started_at DESC LIMIT 50",
        None => "SELECT id, instance_id, started_at, ended_at, duration_secs, launch_method, cpu_avg_percent, ram_avg_mb, ram_peak_mb, status, notes FROM sessions ORDER BY started_at DESC LIMIT 50",
    };

    let mut stmt = conn.prepare(query).map_err(|e| format!("DB error: {}", e))?;

    let params: Vec<Box<dyn rusqlite::types::ToSql>> = match instance_id {
        Some(id) => vec![Box::new(id.to_string())],
        None => vec![],
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let sessions = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Session {
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
    })
    .map_err(|e| format!("DB query error: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(sessions)
}

pub fn generate_report(db: &Database, session_id: &str) -> Result<SessionReport, String> {
    let session = get_session(db, session_id)?;

    let conn = db.conn();
    let rec_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM recommendations WHERE session_id = ?1",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let obs_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM process_observations WHERE session_id = ?1",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    drop(conn);
    let summaries = telemetry::get_summaries(db, session_id).unwrap_or_default();
    let telemetry_points = summaries.len();

    let fps_values: Vec<f32> = summaries.iter().filter_map(|s| s.fps_avg).collect();
    let avg_fps = if fps_values.is_empty() {
        None
    } else {
        Some(fps_values.iter().sum::<f32>() / fps_values.len() as f32)
    };

    let low_1pct_values: Vec<f32> = summaries.iter().filter_map(|s| s.fps_low_1pct).collect();
    let low_1pct_fps = if low_1pct_values.is_empty() {
        None
    } else {
        let mut sorted = low_1pct_values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = (sorted.len() as f32 * 0.01).ceil().max(1.0) as usize;
        Some(sorted[..idx.min(sorted.len())].iter().sum::<f32>() / idx.min(sorted.len()) as f32)
    };

    let tps_values: Vec<f32> = summaries.iter().filter_map(|s| s.tps_avg).collect();
    let avg_tps = if tps_values.is_empty() {
        None
    } else {
        Some(tps_values.iter().sum::<f32>() / tps_values.len() as f32)
    };

    let duration_part = session
        .duration_secs
        .map(|d| format!(" Duration: {}m {}s.", d / 60, d % 60))
        .unwrap_or_default();

    let perf_part = match (session.cpu_avg_percent, session.ram_avg_mb) {
        (Some(cpu), Some(ram)) => format!(" Avg CPU: {:.1}%, Avg RAM: {:.0} MB.", cpu, ram),
        _ => String::new(),
    };

    let fps_part = avg_fps
        .map(|f| format!(" Avg FPS: {:.0}.", f))
        .unwrap_or_default();

    let summary = format!(
        "Session for instance {}.{}{}{} {} recommendations, {} process observations, {} telemetry points.",
        session.instance_id,
        duration_part,
        perf_part,
        fps_part,
        rec_count,
        obs_count,
        telemetry_points,
    );

    Ok(SessionReport {
        session,
        recommendations_applied: rec_count as usize,
        process_observations: obs_count as usize,
        avg_fps,
        low_1pct_fps,
        avg_tps,
        telemetry_points,
        summary,
    })
}

fn store_hardware_snapshot(conn: &rusqlite::Connection, session_id: &str) {
    let hw = hardware::collect_hardware_info();
    let device_id = {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(hw.hostname.as_bytes());
        hasher.update(hw.cpu_model.as_bytes());
        format!("dev-{}", hex::encode(&hasher.finalize()[..8]))
    };
    conn.execute(
        "INSERT INTO hardware_snapshots (id, session_id, device_id, cpu_usage, ram_total_mb, ram_used_mb, ram_available_mb, gpu_model) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            session_id,
            device_id,
            hw.cpu_usage_percent as f64,
            hw.ram_total_mb as i64,
            hw.ram_used_mb as i64,
            hw.ram_available_mb as i64,
            hw.gpu_model,
        ],
    ).ok();
}
