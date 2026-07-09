use crate::db::Database;
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

    Ok(session)
}

pub fn end_session(db: &Database, session_id: &str) -> Result<Session, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.conn();

    conn.execute(
        "UPDATE sessions SET ended_at = ?1, status = 'completed' WHERE id = ?2",
        rusqlite::params![now, session_id],
    ).map_err(|e| format!("Failed to end session: {}", e))?;

    get_session(db, session_id)
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

pub fn list_sessions(db: &Database, instance_id: Option<&str>) -> Vec<Session> {
    let conn = db.conn();

    let query = match instance_id {
        Some(_) => "SELECT id, instance_id, started_at, ended_at, duration_secs, launch_method, cpu_avg_percent, ram_avg_mb, ram_peak_mb, status, notes FROM sessions WHERE instance_id = ?1 ORDER BY started_at DESC LIMIT 50",
        None => "SELECT id, instance_id, started_at, ended_at, duration_secs, launch_method, cpu_avg_percent, ram_avg_mb, ram_peak_mb, status, notes FROM sessions ORDER BY started_at DESC LIMIT 50",
    };

    let mut stmt = conn.prepare(query).unwrap();

    let params: Vec<Box<dyn rusqlite::types::ToSql>> = match instance_id {
        Some(id) => vec![Box::new(id.to_string())],
        None => vec![],
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    stmt.query_map(param_refs.as_slice(), |row| {
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
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
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

    let summary = format!(
        "Session for instance {} started at {}. {} recommendations generated, {} process observations recorded.",
        session.instance_id,
        session.started_at,
        rec_count,
        obs_count
    );

    Ok(SessionReport {
        session,
        recommendations_applied: rec_count as usize,
        process_observations: obs_count as usize,
        summary,
    })
}
