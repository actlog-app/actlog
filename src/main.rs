use actlog::db::schema::init_database;
use actlog::tracker::{SharedTrackerState, TrackerEngine};
use actlog::ui::app::ActLogApp;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;

fn resolve_db_path() -> String {
    let args: Vec<String> = std::env::args().collect();
    for i in 1..args.len() {
        if args[i] == "--db-path" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
        if args[i] == "--portable" {
            return "actlog.db".to_string();
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = PathBuf::from(appdata).join("ActLog");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("actlog.db").to_string_lossy().to_string();
    }

    "actlog.db".to_string()
}

fn main() -> iced::Result {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::HiDpi::*;
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let args: Vec<String> = std::env::args().collect();
    for arg in &args[1..] {
        if arg == "-v" || arg == "--version" {
            println!("ActLog v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
    }

    let mut screenshot_path = None;
    for i in 1..args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            screenshot_path = Some(args[i + 1].clone());
        }
    }

    let db_path = resolve_db_path();

    // 1. Initialize SQLite Database
    {
        let conn = Connection::open(&db_path).expect("Failed to open or create SQLite database");
        init_database(&conn).expect("Failed to initialize database schema");
    }

    // 2. Shared State
    let state = Arc::new(SharedTrackerState::default());
    let tracker_state = state.clone();

    // 3. Start Background Tracker Engine
    let tracker = TrackerEngine::new(state.clone(), db_path.clone());
    let tracker_handle = tracker.start_loop();

    // 4. Mount Desktop Window UI via Iced
    let state_for_app = state.clone();
    let db_path_for_app = db_path.clone();

    let app = iced::application(
        move || ActLogApp::new(state_for_app.clone(), db_path_for_app.clone(), screenshot_path.clone()),
        ActLogApp::update,
        ActLogApp::view,
    )
    .title("PT Statistics View")
    .default_font(iced::Font::with_name("Segoe UI"))
    .window_size(iced::Size::new(1024.0, 560.0))
    .subscription(ActLogApp::subscription)
    .theme(ActLogApp::theme);

    let res = app.run();

    // 5. Clean shutdown
    tracker_state.is_running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = tracker_handle.join();

    res
}
