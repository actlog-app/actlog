use actlog::db::schema::init_database;
use actlog::server::http::HttpServer;
use actlog::tracker::engine::{SharedTrackerState, TrackerEngine};
use actlog::ui::app::ActLogApp;
use eframe::egui;
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

fn main() -> eframe::Result {
    let db_path = resolve_db_path();

    // 1. Initialize SQLite Database
    {
        let conn = Connection::open(&db_path).expect("Failed to open or create SQLite database");
        init_database(&conn).expect("Failed to initialize database schema");
    }

    // 2. Shared State
    let state = Arc::new(SharedTrackerState::default());

    // 3. Start Background Tracker Engine
    let tracker = TrackerEngine::new(state.clone(), db_path.clone());
    let _tracker_handle = tracker.start_loop();

    // 4. Start Local Ingestion & Integration HTTP Server (default: 5577)
    let server = HttpServer::new(state.clone(), db_path.clone(), 5577);
    let _server_handle = server.start();

    // 5. Mount Desktop Window UI
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 680.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("ActLog - Windows Activity Tracker"),
        ..Default::default()
    };

    eframe::run_native(
        "ActLog",
        native_options,
        Box::new(move |_cc| Ok(Box::new(ActLogApp::new(state, db_path)))),
    )
}
