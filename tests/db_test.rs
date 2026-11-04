use actlog::db::models::{ActivitySession, MatchField};
use actlog::db::repository::Repository;
use actlog::db::schema::init_database;
use rusqlite::Connection;

#[test]
fn test_schema_creation() {
    let conn = Connection::open_in_memory().expect("open memory db");
    init_database(&conn).expect("init db");

    let tags = Repository::get_all_tags(&conn).expect("get tags");
    assert!(!tags.is_empty(), "default tags should be seeded");
    assert!(tags.iter().any(|t| t.name == "work"));
    assert!(tags.iter().any(|t| t.name == "games"));

    let rules = Repository::get_all_rules(&conn).expect("get rules");
    assert!(!rules.is_empty(), "default rules should be seeded");
    assert!(rules.iter().any(|r| r.pattern == "code.exe" && r.match_field == MatchField::AppName));
}

#[test]
fn test_session_aggregation_and_flush() {
    let conn = Connection::open_in_memory().expect("open memory db");
    init_database(&conn).expect("init db");

    // First tick at t=100
    let session1 = ActivitySession {
        id: None,
        start_time: 100,
        end_time: 101,
        duration: 1,
        app_name: "code.exe".to_string(),
        exe_path: Some("C:\\VSCode\\code.exe".to_string()),
        window_title: "main.rs - actlog".to_string(),
        url: None,
        domain: None,
        tag_id: Some(1),
        is_afk: false,
    };
    let id1 = Repository::save_or_merge_session(&conn, &session1).expect("save tick 1");

    // Second tick at t=102 (within 10s gap, identical context)
    let session2 = ActivitySession {
        id: None,
        start_time: 102,
        end_time: 103,
        duration: 1,
        app_name: "code.exe".to_string(),
        exe_path: Some("C:\\VSCode\\code.exe".to_string()),
        window_title: "main.rs - actlog".to_string(),
        url: None,
        domain: None,
        tag_id: Some(1),
        is_afk: false,
    };
    let id2 = Repository::save_or_merge_session(&conn, &session2).expect("save tick 2");
    assert_eq!(id1, id2, "Consecutive matching ticks must merge into the same row");

    // Check merged row duration
    let activities = Repository::get_day_activities(&conn, 50, 150).expect("get activities");
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].start_time, 100);
    assert_eq!(activities[0].end_time, 103);
    assert_eq!(activities[0].duration, 3);

    // Third tick with different window title -> must insert new row
    let session3 = ActivitySession {
        id: None,
        start_time: 104,
        end_time: 105,
        duration: 1,
        app_name: "code.exe".to_string(),
        exe_path: Some("C:\\VSCode\\code.exe".to_string()),
        window_title: "lib.rs - actlog".to_string(),
        url: None,
        domain: None,
        tag_id: Some(1),
        is_afk: false,
    };
    let id3 = Repository::save_or_merge_session(&conn, &session3).expect("save tick 3");
    assert_ne!(id1, id3, "Changed window title must create new row");

    let all = Repository::get_day_activities(&conn, 50, 150).expect("get all");
    assert_eq!(all.len(), 2);

    // Summary test
    let summary = Repository::get_day_summary(&conn, "2026-11-04", 50, 150).expect("get summary");
    assert_eq!(summary.total_active_seconds, 4); // 3s + 1s
    assert_eq!(summary.top_apps.len(), 1);
    assert_eq!(summary.top_apps[0].app_name, "code.exe");
}

