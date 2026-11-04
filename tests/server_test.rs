use actlog::db::models::ActivitySession;
use actlog::server::handlers::ApiHandler;
use actlog::tracker::engine::SharedTrackerState;
use std::sync::Arc;

#[test]
fn test_browser_heartbeat_ingest() {
    let state = Arc::new(SharedTrackerState::default());
    let handler = ApiHandler::new(state.clone(), ":memory:".to_string());

    let payload = r#"{
        "url": "https://github.com/rust-lang/rust",
        "domain": "github.com",
        "title": "rust-lang/rust: Empowering everyone...",
        "tab_active": true
    }"#;

    let response = handler.handle_browser_heartbeat(payload.as_bytes());
    assert_eq!(response.status_code().0, 200);

    let tab_lock = state.current_browser_tab.lock().unwrap();
    assert!(tab_lock.is_some());
    let tab = tab_lock.as_ref().unwrap();
    assert_eq!(tab.domain, "github.com");
    assert_eq!(tab.url, "https://github.com/rust-lang/rust");
}

#[test]
fn test_live_activity_api() {
    let state = Arc::new(SharedTrackerState::default());
    let handler = ApiHandler::new(state.clone(), ":memory:".to_string());

    {
        let mut lock = state.current_activity.lock().unwrap();
        *lock = Some(ActivitySession {
            id: Some(42),
            start_time: 1000,
            end_time: 1240,
            duration: 240,
            app_name: "vlc.exe".to_string(),
            exe_path: Some("C:\\Program Files\\VideoLAN\\VLC\\vlc.exe".to_string()),
            window_title: "[SubsPlease] Frieren - 28.mkv".to_string(),
            url: None,
            domain: None,
            tag_id: Some(4),
            is_afk: false,
        });
    }

    let response = handler.handle_current_activity();
    assert_eq!(response.status_code().0, 200);
}
