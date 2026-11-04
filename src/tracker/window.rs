#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveWindowInfo {
    pub app_name: String,
    pub exe_path: Option<String>,
    pub window_title: String,
}

#[cfg(windows)]
pub mod win32 {
    use super::ActiveWindowInfo;
    use std::path::Path;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumChildWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId,
    };

    struct ProcIds {
        target_pid: u32,
    }

    unsafe extern "system" fn enum_child_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let pids = &mut *(lparam.0 as *mut ProcIds);
        let mut child_pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut child_pid));
        if child_pid != 0 && child_pid != pids.target_pid {
            pids.target_pid = child_pid;
            return BOOL(0); // Stop enumeration
        }
        BOOL(1) // Continue
    }

    pub fn get_active_window() -> Option<ActiveWindowInfo> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            // 1. Get Window Title
            let title_len = GetWindowTextLengthW(hwnd);
            let mut title_buf = vec![0u16; (title_len + 1) as usize];
            let read_len = GetWindowTextW(hwnd, &mut title_buf);
            let window_title = String::from_utf16_lossy(&title_buf[..read_len as usize])
                .trim()
                .to_string();

            // 2. Get Process ID
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return None;
            }

            // Resolve process path (handles UWP ApplicationFrameHost wrapper)
            let (app_name, exe_path) = resolve_process_info(hwnd, pid);

            // Filter out lock screen and own process
            if app_name.eq_ignore_ascii_case("lockapp.exe")
                || app_name.eq_ignore_ascii_case("actlog.exe")
            {
                return None;
            }

            Some(ActiveWindowInfo {
                app_name,
                exe_path,
                window_title,
            })
        }
    }

    unsafe fn resolve_process_info(hwnd: HWND, mut pid: u32) -> (String, Option<String>) {
        for _ in 0..2 {
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut buf = vec![0u16; 1024];
                let mut size = buf.len() as u32;

                let success = QueryFullProcessImageNameW(
                    handle,
                    PROCESS_NAME_FORMAT(0),
                    PWSTR(buf.as_mut_ptr()),
                    &mut size,
                );

                let _ = windows::Win32::Foundation::CloseHandle(handle);

                if success.is_ok() && size > 0 {
                    let full_path = String::from_utf16_lossy(&buf[..size as usize]);
                    let path = Path::new(&full_path);
                    let file_name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown.exe".to_string());

                    // Check if it's UWP ApplicationFrameHost wrapper
                    if file_name.eq_ignore_ascii_case("applicationframehost.exe")
                        || file_name.eq_ignore_ascii_case("wwahost.exe")
                    {
                        let mut pids = ProcIds { target_pid: pid };
                        let _ = EnumChildWindows(
                            hwnd,
                            Some(enum_child_proc),
                            LPARAM(&mut pids as *mut _ as isize),
                        );
                        if pids.target_pid != pid {
                            pid = pids.target_pid;
                            continue;
                        }
                    }

                    return (file_name, Some(full_path));
                }
            }
        }

        ("unknown.exe".to_string(), None)
    }
}

#[cfg(not(windows))]
pub mod win32 {
    use super::ActiveWindowInfo;

    pub fn get_active_window() -> Option<ActiveWindowInfo> {
        Some(ActiveWindowInfo {
            app_name: "mock.exe".to_string(),
            exe_path: Some("/bin/mock.exe".to_string()),
            window_title: "Mock Window Title".to_string(),
        })
    }
}

pub fn get_foreground_window() -> Option<ActiveWindowInfo> {
    win32::get_active_window()
}
