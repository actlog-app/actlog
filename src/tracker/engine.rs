use crate::db::models::{ActivitySession, BrowserHeartbeat, RawTick};
use crate::db::repository::Repository;
use crate::tracker::gamepad::poll_gamepads;
use crate::tracker::idle::get_user_idle_ms;
use crate::tracker::rules::RuleEngine;
use crate::tracker::window::get_foreground_window;
use chrono::Utc;
use rusqlite::Connection;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct SharedTrackerState {
    pub current_browser_tab: Mutex<Option<BrowserHeartbeat>>,
    pub current_activity: Mutex<Option<ActivitySession>>,
    pub is_tracking_paused: AtomicBool,
}

pub struct TrackerEngine {
    state: Arc<SharedTrackerState>,
    db_path: String,
    afk_threshold_ms: u64,
}

impl TrackerEngine {
    pub fn new(state: Arc<SharedTrackerState>, db_path: String) -> Self {
        Self {
            state,
            db_path,
            afk_threshold_ms: 180_000, // 3 minutes default
        }
    }

    pub fn start_loop(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let conn = match Connection::open(&self.db_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to open DB for tracker engine: {}", e);
                    return;
                }
            };

            let mut rules = Repository::get_all_rules(&conn).unwrap_or_default();
            let mut last_rule_reload = Utc::now().timestamp();
            let mut last_flush_time = Utc::now().timestamp();

            let mut active_session: Option<ActivitySession> = None;

            loop {
                thread::sleep(Duration::from_millis(1000));

                if self.state.is_tracking_paused.load(Ordering::Relaxed) {
                    continue;
                }

                let now_ts = Utc::now().timestamp();

                // Reload rules every 30 seconds
                if now_ts - last_rule_reload > 30 {
                    if let Ok(new_rules) = Repository::get_all_rules(&conn) {
                        rules = new_rules;
                    }
                    last_rule_reload = now_ts;
                }

                // 1. Poll Gamepad & Idle
                let _gamepad_active = poll_gamepads();
                let mouse_kb_idle_ms = get_user_idle_ms();
                let is_afk = mouse_kb_idle_ms > self.afk_threshold_ms;

                // 2. Poll Foreground Window
                let window_info = get_foreground_window();
                let (app_name, exe_path, window_title) = match window_info {
                    Some(w) => (w.app_name, w.exe_path, w.window_title),
                    None => continue,
                };

                // 3. Correlate browser info if foreground app is a browser
                let is_browser = is_browser_app(&app_name);
                let (url, domain) = if is_browser {
                    let tab_guard = self.state.current_browser_tab.lock().unwrap();
                    if let Some(ref tab) = *tab_guard {
                        (Some(tab.url.clone()), Some(tab.domain.clone()))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                // 4. Create Raw Tick
                let tick = RawTick {
                    timestamp: now_ts,
                    app_name: app_name.clone(),
                    exe_path,
                    window_title: window_title.clone(),
                    url: url.clone(),
                    domain: domain.clone(),
                    is_afk,
                    gamepad_active: false,
                };

                // 5. Categorize via Rule Engine
                let tag_id = RuleEngine::match_tag(&tick, &rules);

                // 6. Accumulate into active session
                let should_continue = if let Some(ref current) = active_session {
                    current.app_name == tick.app_name
                        && current.window_title == tick.window_title
                        && current.url == tick.url
                        && current.tag_id == tag_id
                        && current.is_afk == tick.is_afk
                        && (now_ts - current.end_time) <= 5
                } else {
                    false
                };

                if should_continue {
                    if let Some(ref mut current) = active_session {
                        current.end_time = now_ts;
                        current.duration = current.end_time - current.start_time;
                    }
                } else {
                    // Flush previous session to DB
                    if let Some(finished) = active_session.take() {
                        let _ = Repository::save_or_merge_session(&conn, &finished);
                    }

                    // Start new session
                    active_session = Some(ActivitySession {
                        id: None,
                        start_time: now_ts,
                        end_time: now_ts + 1,
                        duration: 1,
                        app_name,
                        exe_path: tick.exe_path,
                        window_title,
                        url,
                        domain,
                        tag_id,
                        is_afk,
                    });
                }

                // Checkpoint flush every 60 seconds
                if now_ts - last_flush_time > 60 {
                    if let Some(ref current) = active_session {
                        let _ = Repository::save_or_merge_session(&conn, current);
                    }
                    last_flush_time = now_ts;
                }

                // Update shared state for UI and API reads
                {
                    let mut lock = self.state.current_activity.lock().unwrap();
                    *lock = active_session.clone();
                }
            }
        })
    }
}

fn is_browser_app(app_name: &str) -> bool {
    let lower = app_name.to_lowercase();
    lower.contains("chrome")
        || lower.contains("msedge")
        || lower.contains("firefox")
        || lower.contains("brave")
        || lower.contains("opera")
}
