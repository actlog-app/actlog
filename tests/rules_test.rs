use actlog::model::{MatchField, RawTick, Rule};
use actlog::tracker::rules::RuleEngine;

#[test]
fn test_rule_matching_priority_and_types() {
    let rules = vec![
        Rule {
            id: Some(1),
            pattern: "youtube.com".to_string(),
            match_field: MatchField::Domain,
            tag_id: 10, // Media
            priority: 100,
        },
        Rule {
            id: Some(2),
            pattern: "code.exe".to_string(),
            match_field: MatchField::AppName,
            tag_id: 20, // Dev
            priority: 90,
        },
        Rule {
            id: Some(3),
            pattern: ".*\\.rs - VS Code$".to_string(),
            match_field: MatchField::WindowTitle,
            tag_id: 30, // Rust Focus
            priority: 80,
        },
    ];

    // Tick 1: Chrome on YouTube
    let tick1 = RawTick {
        timestamp: 1000,
        app_name: "chrome.exe".to_string(),
        exe_path: None,
        window_title: "YouTube - Video".to_string(),
        url: Some("https://youtube.com/watch?v=123".to_string()),
        domain: Some("youtube.com".to_string()),
        is_afk: false,
        gamepad_active: false,
    };
    assert_eq!(RuleEngine::match_tag(&tick1, &rules), Some(10));

    // Tick 2: VS Code
    let tick2 = RawTick {
        timestamp: 1001,
        app_name: "code.exe".to_string(),
        exe_path: None,
        window_title: "main.rs - VS Code".to_string(),
        url: None,
        domain: None,
        is_afk: false,
        gamepad_active: false,
    };
    assert_eq!(RuleEngine::match_tag(&tick2, &rules), Some(20));

    // Tick 3: Unknown app
    let tick3 = RawTick {
        timestamp: 1002,
        app_name: "calc.exe".to_string(),
        exe_path: None,
        window_title: "Calculator".to_string(),
        url: None,
        domain: None,
        is_afk: false,
        gamepad_active: false,
    };
    assert_eq!(RuleEngine::match_tag(&tick3, &rules), None);
}

