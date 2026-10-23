use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingSound {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub rank: u32,
    pub user_count: u64,
    pub play_url: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingHashtag {
    pub tag: String,
    pub usage_count: u64,
    pub platform: String,
}
