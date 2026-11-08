pub mod gamepad;
pub mod idle;
pub mod window;

pub use gamepad::{last_gamepad_activity_ms, poll_gamepads};
pub use idle::get_user_idle_ms;
pub use window::{get_foreground_window, is_foreground_fullscreen, ActiveWindowInfo};
