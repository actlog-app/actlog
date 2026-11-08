use std::sync::atomic::AtomicU64;

static LAST_GAMEPAD_ACTIVITY_TICK: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
pub mod win32 {
    use super::LAST_GAMEPAD_ACTIVITY_TICK;
    use std::sync::atomic::Ordering;
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE};

    // Standard XInput thresholds from Microsoft DirectInput SDK
    const LEFT_THUMB_DEADZONE: i16 = 7849;
    const RIGHT_THUMB_DEADZONE: i16 = 8689;
    const TRIGGER_THRESHOLD: u8 = 30;

    fn in_deadzone(val: i16, deadzone: i16) -> bool {
        val > -deadzone && val < deadzone
    }

    pub fn poll_gamepad_activity() -> bool {
        let mut any_active = false;

        unsafe {
            for i in 0..4 {
                let mut state = XINPUT_STATE::default();
                if XInputGetState(i, &mut state) == 0 {
                    // ERROR_SUCCESS == 0
                    let gp = state.Gamepad;
                    let stick_active = !in_deadzone(gp.sThumbLX, LEFT_THUMB_DEADZONE)
                        || !in_deadzone(gp.sThumbLY, LEFT_THUMB_DEADZONE)
                        || !in_deadzone(gp.sThumbRX, RIGHT_THUMB_DEADZONE)
                        || !in_deadzone(gp.sThumbRY, RIGHT_THUMB_DEADZONE);

                    let trigger_active = gp.bLeftTrigger > TRIGGER_THRESHOLD
                        || gp.bRightTrigger > TRIGGER_THRESHOLD;

                    let button_active = gp.wButtons.0 != 0;

                    if stick_active || trigger_active || button_active {
                        let now = GetTickCount() as u64;
                        LAST_GAMEPAD_ACTIVITY_TICK.store(now, Ordering::Relaxed);
                        any_active = true;
                    }
                }
            }
        }

        any_active
    }

    pub fn get_last_gamepad_tick() -> u64 {
        LAST_GAMEPAD_ACTIVITY_TICK.load(Ordering::Relaxed)
    }
}

#[cfg(not(windows))]
pub mod win32 {
    pub fn poll_gamepad_activity() -> bool {
        false
    }
    pub fn get_last_gamepad_tick() -> u64 {
        0
    }
}

pub fn poll_gamepads() -> bool {
    win32::poll_gamepad_activity()
}

pub fn last_gamepad_activity_ms() -> u64 {
    win32::get_last_gamepad_tick()
}
