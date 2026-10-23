use crate::db::Database;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub session_id: Option<String>,
    pub game_id: Option<String>,
    pub source_video: String,
    pub clip_path: Option<String>,
    pub start_time_ms: i64,
    pub end_time_ms: i64,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub composite_score: Option<f64>,
    pub decima_confidence: Option<f64>,
    pub thumbnail_path: Option<String>,
    pub title: Option<String>,
    pub transcript: Option<String>,
    pub rationale: Option<String>,
    pub mood_scores: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSignal {
    pub id: String,
    pub clip_id: String,
    pub signal_type: String,
    pub signal_value: f64,
    pub signal_data: Option<String>,
    pub timestamp_ms: i64,
}

pub fn create_clip(db: &Database, clip: &Clip) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO clips (id, session_id, game_id, source_video, clip_path, start_time_ms, end_time_ms, duration_ms, status, composite_score, decima_confidence, thumbnail_path, title, transcript, rationale, mood_scores)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        rusqlite::params![
            clip.id, clip.session_id, clip.game_id, clip.source_video, clip.clip_path,
            clip.start_time_ms, clip.end_time_ms, clip.duration_ms, clip.status,
            clip.composite_score, clip.decima_confidence, clip.thumbnail_path,
            clip.title, clip.transcript, clip.rationale, clip.mood_scores,
        ],
    ).map_err(|e| format!("Failed to create clip: {}", e))?;
    Ok(())
}

pub fn get_clip(db: &Database, clip_id: &str) -> Result<Clip, String> {
    let conn = db.conn()?;
    conn.query_row(
        "SELECT id, session_id, game_id, source_video, clip_path, start_time_ms, end_time_ms, duration_ms, status, composite_score, decima_confidence, thumbnail_path, title, transcript, rationale, mood_scores, created_at FROM clips WHERE id = ?1",
        rusqlite::params![clip_id],
        |row| Ok(Clip {
            id: row.get(0)?,
            session_id: row.get(1)?,
            game_id: row.get(2)?,
            source_video: row.get(3)?,
            clip_path: row.get(4)?,
            start_time_ms: row.get(5)?,
            end_time_ms: row.get(6)?,
            duration_ms: row.get(7)?,
            status: row.get(8)?,
            composite_score: row.get(9)?,
            decima_confidence: row.get(10)?,
            thumbnail_path: row.get(11)?,
            title: row.get(12)?,
            transcript: row.get(13)?,
            rationale: row.get(14)?,
            mood_scores: row.get(15)?,
            created_at: row.get(16)?,
        }),
    ).map_err(|e| format!("Clip not found: {}", e))
}

pub fn list_clips(db: &Database, game_id: Option<&str>, status: Option<&str>, limit: u32) -> Result<Vec<Clip>, String> {
    let conn = db.conn()?;
    let mut conditions = vec!["1=1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(gid) = game_id {
        conditions.push(format!("game_id = ?{}", params.len() + 1));
        params.push(Box::new(gid.to_string()));
    }
    if let Some(s) = status {
        conditions.push(format!("status = ?{}", params.len() + 1));
        params.push(Box::new(s.to_string()));
    }

    let query = format!(
        "SELECT id, session_id, game_id, source_video, clip_path, start_time_ms, end_time_ms, duration_ms, status, composite_score, decima_confidence, thumbnail_path, title, transcript, rationale, mood_scores, created_at FROM clips WHERE {} ORDER BY created_at DESC LIMIT {}",
        conditions.join(" AND "),
        limit,
    );

    let mut stmt = conn.prepare(&query).map_err(|e| format!("DB error: {}", e))?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let clips = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Clip {
            id: row.get(0)?,
            session_id: row.get(1)?,
            game_id: row.get(2)?,
            source_video: row.get(3)?,
            clip_path: row.get(4)?,
            start_time_ms: row.get(5)?,
            end_time_ms: row.get(6)?,
            duration_ms: row.get(7)?,
            status: row.get(8)?,
            composite_score: row.get(9)?,
            decima_confidence: row.get(10)?,
            thumbnail_path: row.get(11)?,
            title: row.get(12)?,
            transcript: row.get(13)?,
            rationale: row.get(14)?,
            mood_scores: row.get(15)?,
            created_at: row.get(16)?,
        })
    })
    .map_err(|e| format!("Query error: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(clips)
}

pub fn update_clip_status(db: &Database, clip_id: &str, status: &str) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE clips SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, clip_id],
    ).map_err(|e| format!("Failed to update clip: {}", e))?;
    Ok(())
}

pub fn store_clip_signal(db: &Database, signal: &ClipSignal) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO clip_signals (id, clip_id, signal_type, signal_value, signal_data, timestamp_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![signal.id, signal.clip_id, signal.signal_type, signal.signal_value, signal.signal_data, signal.timestamp_ms],
    ).map_err(|e| format!("Failed to store signal: {}", e))?;
    Ok(())
}
