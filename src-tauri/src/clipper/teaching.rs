use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipFeedback {
    pub id: String,
    pub clip_id: String,
    pub action: String,
    pub rating: Option<i32>,
    pub rationale_agreed: Option<bool>,
    pub written_reason: Option<String>,
    pub feedback_at: String,
}

pub fn store_feedback(db: &crate::db::Database, feedback: &ClipFeedback) -> Result<(), String> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO clip_feedback (id, clip_id, action, rating, rationale_agreed, written_reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            feedback.id,
            feedback.clip_id,
            feedback.action,
            feedback.rating,
            feedback.rationale_agreed,
            feedback.written_reason,
        ],
    )
    .map_err(|e| format!("Failed to store feedback: {}", e))?;
    Ok(())
}
