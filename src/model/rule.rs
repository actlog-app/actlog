use serde::{Deserialize, Serialize};

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
