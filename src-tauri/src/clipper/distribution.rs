use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostizConfig {
    pub instance_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionEntry {
    pub id: String,
    pub clip_id: String,
    pub platform: String,
    pub post_id: Option<String>,
    pub status: String,
    pub scheduled_at: Option<String>,
    pub posted_at: Option<String>,
    pub postiz_post_id: Option<String>,
    pub engagement_data: Option<String>,
}
