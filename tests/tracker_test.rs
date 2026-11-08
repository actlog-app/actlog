use actlog::db::repository::Repository;
use actlog::db::schema::init_database;
use actlog::model::MatchField;
use actlog::platform::is_foreground_fullscreen;
use actlog::tracker::SharedTrackerState;
use rusqlite::Connection;
use std::sync::atomic::Ordering;

#[test]
fn test_shared_tracker_state_lifecycle() {
    let state = SharedTrackerState::default();
    assert!(state.is_running.load(Ordering::Relaxed));
    assert!(!state.is_tracking_paused.load(Ordering::Relaxed));

    // Test pause toggle
    state.is_tracking_paused.fetch_xor(true, Ordering::Relaxed);
    assert!(state.is_tracking_paused.load(Ordering::Relaxed));

    // Test shutdown signal
    state.is_running.store(false, Ordering::Relaxed);
    assert!(!state.is_running.load(Ordering::Relaxed));
}

#[test]
fn test_fullscreen_query_does_not_panic() {
    // Should run safely without panic on any platform
    let _ = is_foreground_fullscreen();
}

#[test]
fn test_tag_assignment_and_rule_persistence() {
    let conn = Connection::open_in_memory().expect("open memory db");
    init_database(&conn).expect("init db");

    let tags = Repository::get_all_tags(&conn).expect("get tags");
    let work_tag = tags.iter().find(|t| t.name == "work").expect("work tag");
    let work_tag_id = work_tag.id.unwrap();

    // Insert new rule via the app tag-assignment logic
    conn.execute(
        "INSERT INTO rules (pattern, match_field, tag_id, priority) VALUES (?1, 'app_name', ?2, 20)",
        ("blender.exe", work_tag_id),
    ).expect("insert rule");

    let rules = Repository::get_all_rules(&conn).expect("get rules");
    let rule = rules.iter().find(|r| r.pattern == "blender.exe").expect("found blender rule");
    assert_eq!(rule.match_field, MatchField::AppName);
    assert_eq!(rule.tag_id, work_tag_id);
    assert_eq!(rule.priority, 20);
}
