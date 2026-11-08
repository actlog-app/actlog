use actlog::db::schema::init_database;
// use actlog::server::http::HttpServer;
use actlog::tracker::{SharedTrackerState, TrackerEngine};
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
    let args: Vec<String> = std::env::args().collect();
    for arg in &args[1..] {
        if arg == "-v" || arg == "--version" {
            println!("ActLog v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
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

    // 4. Local server separated for now (focusing 100% on desktop app)
    // let server = HttpServer::new(state.clone(), db_path.clone(), 5577);
    // let _server_handle = server.start();

    // 5. Mount Desktop Window UI (Modern Focus Dashboard)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 720.0])
            .with_min_inner_size([750.0, 500.0])
            .with_title("ActLog - Focus & Activity Dashboard"),
        ..Default::default()
    };

    println!("Starting eframe window...");
    let res = eframe::run_native(
        "ActLog",
        native_options,
        Box::new(move |_cc| {
            println!("eframe app setup callback running!");
            Ok(Box::new(ActLogApp::new(state, db_path)))
        }),
    );
    println!("eframe::run_native returned: {:?}", res);

    // Clean shutdown: signal tracker loop to flush and exit
    tracker_state.is_running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = tracker_handle.join();

    res
}
