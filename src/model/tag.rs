use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
    pub productivity_score: i32, // -2 (high procrastination) to +2 (high focus)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTotal {
    pub tag_name: String,
    pub color: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTotal {
    pub app_name: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaySummary {
    pub date: String, // YYYY-MM-DD
    pub total_active_seconds: i64,
    pub total_afk_seconds: i64,
    pub top_apps: Vec<AppTotal>,
    pub categories: Vec<CategoryTotal>,
}
