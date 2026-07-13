use crate::db::Database;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct RollbackPoint {
    pub id: String,
    pub recommendation_id: String,
    pub file_path: String,
    pub original_hash: String,
    pub backup_path: String,
    pub created_at: String,
}

pub fn backup_file(file_path: &Path, recommendation_id: &str) -> Result<RollbackPoint, String> {
    let content = std::fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = hex::encode(hasher.finalize());

    let backup_dir = file_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(".gamepilot_backups");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup dir: {}", e))?;

    let backup_name = format!(
        "{}_{}.bak",
        file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let backup_path = backup_dir.join(&backup_name);
    std::fs::write(&backup_path, &content)
        .map_err(|e| format!("Failed to write backup: {}", e))?;

    Ok(RollbackPoint {
        id: uuid::Uuid::new_v4().to_string(),
        recommendation_id: recommendation_id.to_string(),
        file_path: file_path.to_string_lossy().to_string(),
        original_hash: hash,
        backup_path: backup_path.to_string_lossy().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn rollback_file(rollback_point: &RollbackPoint) -> Result<(), String> {
    let backup = Path::new(&rollback_point.backup_path);
    if !backup.exists() {
        return Err("Backup file not found".to_string());
    }

    let content =
        std::fs::read(backup).map_err(|e| format!("Failed to read backup: {}", e))?;

    let target = Path::new(&rollback_point.file_path);
    std::fs::write(target, &content).map_err(|e| format!("Failed to restore file: {}", e))?;

    Ok(())
}

pub fn save_rollback_point(db: &Database, rp: &RollbackPoint) -> Result<(), String> {
    let conn = db.conn();
    conn.execute(
        "INSERT INTO rollback_points (id, recommendation_id, file_path, original_hash, backup_path, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![rp.id, rp.recommendation_id, rp.file_path, rp.original_hash, rp.backup_path, rp.created_at],
    ).map_err(|e| format!("Failed to save rollback point: {}", e))?;
    Ok(())
}

pub fn get_rollback_points(db: &Database, recommendation_id: &str) -> Result<Vec<RollbackPoint>, String> {
    let conn = db.conn();
    let mut stmt = conn
        .prepare("SELECT id, recommendation_id, file_path, original_hash, backup_path, created_at FROM rollback_points WHERE recommendation_id = ?1 AND restored_at IS NULL ORDER BY created_at DESC")
        .map_err(|e| format!("DB error: {}", e))?;

    let points = stmt.query_map(rusqlite::params![recommendation_id], |row| {
        Ok(RollbackPoint {
            id: row.get(0)?,
            recommendation_id: row.get(1)?,
            file_path: row.get(2)?,
            original_hash: row.get(3)?,
            backup_path: row.get(4)?,
            created_at: row.get(5)?,
        })
    })
    .map_err(|e| format!("DB query error: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(points)
}

pub fn apply_config_change(
    file_path: &Path,
    key: &str,
    new_value: &str,
    recommendation_id: &str,
    db: &Database,
) -> Result<RollbackPoint, String> {
    let rp = backup_file(file_path, recommendation_id)?;
    save_rollback_point(db, &rp)?;

    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let mut modified = String::new();
    let mut found = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some((k, _)) = trimmed.split_once('=').or_else(|| trimmed.split_once(':')) {
                if k.trim() == key {
                    let sep = if trimmed.contains('=') { "=" } else { ":" };
                    modified.push_str(&format!("{}{}{}\n", key, sep, new_value));
                    found = true;
                    continue;
                }
            }
        }
        modified.push_str(line);
        modified.push('\n');
    }

    if !found {
        return Err(format!("Key '{}' not found in config file", key));
    }

    std::fs::write(file_path, &modified)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(rp)
}
