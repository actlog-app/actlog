use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
    pub productivity_score: i32, // -2 (high procrastination) to +2 (high focus)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchField {
    AppName,
    WindowTitle,
    Domain,
}

impl MatchField {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchField::AppName => "app_name",
            MatchField::WindowTitle => "window_title",
            MatchField::Domain => "domain",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "app_name" => Some(MatchField::AppName),
            "window_title" => Some(MatchField::WindowTitle),
            "domain" => Some(MatchField::Domain),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rule {
    pub id: Option<i64>,
    pub pattern: String,
    pub match_field: MatchField,
    pub tag_id: i64,
    pub priority: i32,
}

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

