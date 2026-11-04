use crate::db::models::{MatchField, RawTick, Rule};
use regex::Regex;

pub struct RuleEngine;

impl RuleEngine {
    pub fn match_tag(tick: &RawTick, rules: &[Rule]) -> Option<i64> {
        for rule in rules {
            let target_str: Option<&str> = match rule.match_field {
                MatchField::AppName => Some(&tick.app_name),
                MatchField::WindowTitle => Some(&tick.window_title),
                MatchField::Domain => tick.domain.as_deref(),
            };

            if let Some(target) = target_str {
                if Self::is_match(target, &rule.pattern) {
                    return Some(rule.tag_id);
                }
            }
        }
        None
    }

    fn is_match(target: &str, pattern: &str) -> bool {
        // Fast path: Case-insensitive substring match
        if target.to_lowercase().contains(&pattern.to_lowercase()) {
            return true;
        }

        // Regex path (if pattern looks like a regex)
        if pattern.contains('^') || pattern.contains('$') || pattern.contains(".*") {
            if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                return re.is_match(target);
            }
        }

        false
    }
}
