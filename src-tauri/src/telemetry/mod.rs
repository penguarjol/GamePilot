use serde::Serialize;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct LogEvent {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetrySummary {
    pub minute_ts: String,
    pub cpu_avg: Option<f64>,
    pub ram_avg_mb: Option<f64>,
    pub ram_peak_mb: Option<f64>,
    pub hog_count: Option<i32>,
    pub fps_avg: Option<f32>,
    pub fps_low_1pct: Option<f32>,
    pub tps_avg: Option<f32>,
}

/// Tail the Minecraft latest.log file from a given position, returning new lines and the updated position
pub fn tail_minecraft_log(log_path: &Path, from_pos: u64) -> (Vec<LogEvent>, u64) {
    let file = match std::fs::File::open(log_path) {
        Ok(f) => f,
        Err(_) => return (Vec::new(), from_pos),
    };

    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(_) => return (Vec::new(), from_pos),
    };

    let file_len = metadata.len();
    if file_len <= from_pos {
        return (Vec::new(), from_pos);
    }

    let mut reader = BufReader::new(file);
    if from_pos > 0 {
        if reader.seek(SeekFrom::Start(from_pos)).is_err() {
            return (Vec::new(), from_pos);
        }
    }

    let mut events = Vec::new();
    let mut current_pos = from_pos;

    for line in reader.lines() {
        match line {
            Ok(text) => {
                current_pos += text.len() as u64 + 1;
                if let Some(event) = parse_log_line(&text) {
                    events.push(event);
                }
            }
            Err(_) => break,
        }
    }

    let filtered: Vec<LogEvent> = events
        .into_iter()
        .filter(|e| {
            e.level == "ERROR"
                || e.level == "WARN"
                || e.message.contains("heap")
                || e.message.contains("GC")
                || e.message.contains("OutOfMemory")
                || e.message.contains("crash")
                || e.message.contains("Loading complete")
                || e.message.contains("Loaded")
        })
        .collect();

    (filtered, current_pos)
}

fn parse_log_line(line: &str) -> Option<LogEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let timestamp = if trimmed.starts_with('[') {
        trimmed
            .split(']')
            .next()
            .map(|s| s.trim_start_matches('[').to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    let level = if trimmed.contains("/ERROR]") || trimmed.contains("/ERROR}") {
        "ERROR"
    } else if trimmed.contains("/WARN]") || trimmed.contains("/WARN}") {
        "WARN"
    } else if trimmed.contains("/INFO]") || trimmed.contains("/INFO}") {
        "INFO"
    } else {
        "DEBUG"
    };

    Some(LogEvent {
        timestamp,
        level: level.to_string(),
        message: trimmed.to_string(),
    })
}

pub fn extract_fps_from_log(line: &str) -> Option<f32> {
    let lower = line.to_lowercase();

    if lower.contains("fps:") || lower.contains("fps =") {
        let parts: Vec<&str> = lower.split("fps").collect();
        if let Some(after) = parts.get(1) {
            let num_str: String = after
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            return num_str.parse().ok();
        }
    }

    None
}

pub fn extract_tps_from_log(line: &str) -> Option<f32> {
    let lower = line.to_lowercase();

    if lower.contains("tps:") || lower.contains("tps =") {
        let parts: Vec<&str> = lower.split("tps").collect();
        if let Some(after) = parts.get(1) {
            let num_str: String = after
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            return num_str.parse().ok();
        }
    }

    None
}

pub fn find_log_path(instance_path: &Path) -> Option<PathBuf> {
    let candidates = [
        instance_path
            .join(".minecraft")
            .join("logs")
            .join("latest.log"),
        instance_path
            .join("minecraft")
            .join("logs")
            .join("latest.log"),
        instance_path.join("logs").join("latest.log"),
    ];

    candidates.into_iter().find(|p| p.exists())
}

pub fn store_summary(
    db: &crate::db::Database,
    session_id: &str,
    cpu_avg: f64,
    ram_avg: f64,
    ram_peak: f64,
    hog_count: i32,
    fps_avg: Option<f32>,
    fps_low_1pct: Option<f32>,
    tps_avg: Option<f32>,
) -> Result<(), String> {
    let conn = db.conn();
    conn.execute(
        "INSERT INTO telemetry_summaries (id, session_id, minute_ts, cpu_avg, ram_avg_mb, ram_peak_mb, hog_count, fps_avg, fps_low_1pct, tps_avg) \
         VALUES (?1, ?2, datetime('now'), ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            session_id,
            cpu_avg,
            ram_avg,
            ram_peak,
            hog_count,
            fps_avg,
            fps_low_1pct,
            tps_avg,
        ],
    )
    .map_err(|e| format!("Failed to store telemetry summary: {}", e))?;
    Ok(())
}

pub fn get_summaries(
    db: &crate::db::Database,
    session_id: &str,
) -> Result<Vec<TelemetrySummary>, String> {
    let conn = db.conn();
    let mut stmt = conn
        .prepare(
            "SELECT minute_ts, cpu_avg, ram_avg_mb, ram_peak_mb, hog_count, fps_avg, fps_low_1pct, tps_avg \
             FROM telemetry_summaries WHERE session_id = ?1 ORDER BY minute_ts",
        )
        .map_err(|e| format!("DB error: {}", e))?;

    let results = stmt
        .query_map(rusqlite::params![session_id], |row| {
            Ok(TelemetrySummary {
                minute_ts: row.get(0)?,
                cpu_avg: row.get(1)?,
                ram_avg_mb: row.get(2)?,
                ram_peak_mb: row.get(3)?,
                hog_count: row.get(4)?,
                fps_avg: row.get(5)?,
                fps_low_1pct: row.get(6)?,
                tps_avg: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}
