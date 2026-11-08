use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivitySession {
    pub id: Option<i64>,
    pub start_time: i64, // Unix timestamp in seconds
    pub end_time: i64,   // Unix timestamp in seconds
    pub duration: i64,   // Seconds active
    pub app_name: String,
    pub exe_path: Option<String>,
    pub window_title: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub tag_id: Option<i64>,
    pub is_afk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTick {
    pub timestamp: i64,
    pub app_name: String,
    pub exe_path: Option<String>,
    pub window_title: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub is_afk: bool,
    pub gamepad_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserHeartbeat {
    pub url: String,
    pub domain: String,
    pub title: String,
    pub tab_active: bool,
}
