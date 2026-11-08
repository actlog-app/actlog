use crate::model::{ActivitySession, BrowserHeartbeat};
use std::sync::atomic::{AtomicBool, AtomicU16};
use std::sync::Mutex;

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
