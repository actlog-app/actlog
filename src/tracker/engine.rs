use crate::db::models::{ActivitySession, BrowserHeartbeat, RawTick};
use crate::db::repository::Repository;
use crate::tracker::gamepad::poll_gamepads;
use crate::tracker::idle::get_user_idle_ms;
use crate::tracker::rules::RuleEngine;
use crate::tracker::window::get_foreground_window;
use chrono::Utc;
use rusqlite::Connection;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct SharedTrackerState {
    pub current_browser_tab: Mutex<Option<BrowserHeartbeat>>,
    pub current_activity: Mutex<Option<ActivitySession>>,
    pub is_tracking_paused: AtomicBool,
    pub is_running: AtomicBool,
    pub bound_port: AtomicU16,
}

impl Default for SharedTrackerState {
    fn default() -> Self {
        Self {
            current_browser_tab: Mutex::new(None),
            current_activity: Mutex::new(None),
            is_tracking_paused: AtomicBool::new(false),
            is_running: AtomicBool::new(true),
            bound_port: AtomicU16::new(5577),
        }
    }
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
                if !self.state.is_running.load(Ordering::Relaxed) {
                    break;
                }

                thread::sleep(Duration::from_millis(1000));

                if !self.state.is_running.load(Ordering::Relaxed) {
                    break;
                }

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

                // 1. Poll Gamepad, Idle & Fullscreen
                let gamepad_active = poll_gamepads();
                let mouse_kb_idle_ms = get_user_idle_ms();
                let gamepad_idle_ms = if gamepad_active {
                    0
                } else {
                    let last_gp = crate::tracker::gamepad::last_gamepad_activity_ms();
                    if last_gp > 0 {
                        #[cfg(windows)]
                        {
                            let now = unsafe { windows::Win32::System::SystemInformation::GetTickCount() as u64 };
                            now.saturating_sub(last_gp)
                        }
                        #[cfg(not(windows))]
                        {
                            u64::MAX
                        }
                    } else {
                        u64::MAX
                    }
                };

                let effective_idle_ms = mouse_kb_idle_ms.min(gamepad_idle_ms);
                let is_fullscreen = crate::tracker::window::is_foreground_fullscreen();
                let is_afk = if is_fullscreen {
                    false
                } else {
                    effective_idle_ms > self.afk_threshold_ms
                };

                // 2. Poll Foreground Window
                let window_info = get_foreground_window();
                let (app_name, exe_path, window_title) = match window_info {
                    Some(w) => {
                        if w.app_name.eq_ignore_ascii_case("actlog.exe") {
                            // If coming from another app into ActLog, flush that app's session immediately
                            if let Some(finished) = active_session.take() {
                                let _ = Repository::save_or_merge_session(&conn, &finished);
                            }
                            {
                                let mut lock = self.state.current_activity.lock().unwrap();
                                *lock = None;
                            }
                            continue;
                        }
                        (w.app_name, w.exe_path, w.window_title)
                    }
                    None => {
                        if let Some(finished) = active_session.take() {
                            let _ = Repository::save_or_merge_session(&conn, &finished);
                        }
                        {
                            let mut lock = self.state.current_activity.lock().unwrap();
                            *lock = None;
                        }
                        continue;
                    }
                };

                // Cap single AFK session to 1 hour to prevent runaway overnight records
                if is_afk {
                    if let Some(ref current) = active_session {
                        if current.is_afk && (now_ts - current.start_time) >= 3600 {
                            if let Some(finished) = active_session.take() {
                                let _ = Repository::save_or_merge_session(&conn, &finished);
                            }
                            continue;
                        }
                    }
                }

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
                    gamepad_active,
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

            // Flush active session on clean shutdown
            if let Some(finished) = active_session.take() {
                let _ = Repository::save_or_merge_session(&conn, &finished);
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
