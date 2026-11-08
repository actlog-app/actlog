#[cfg(windows)]
pub mod win32 {
    use std::mem::size_of;
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    pub fn get_idle_time_ms() -> u64 {
        unsafe {
            let mut lii = LASTINPUTINFO {
                cbSize: size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };

            if GetLastInputInfo(&mut lii).as_bool() {
                let current_tick = GetTickCount();
                if current_tick >= lii.dwTime {
                    (current_tick - lii.dwTime) as u64
                } else {
                    // Tick count wrapped (happens after ~49.7 days uptime)
                    (u32::MAX - lii.dwTime + current_tick) as u64
                }
            } else {
                0
            }
        }
    }
}

#[cfg(not(windows))]
pub mod win32 {
    pub fn get_idle_time_ms() -> u64 {
        0
    }
}

pub fn get_user_idle_ms() -> u64 {
    win32::get_idle_time_ms()
}
