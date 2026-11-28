use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONULL, MONITORINFO, MonitorFromWindow,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible,
};
use windows::core::PWSTR;

pub struct Foreground {
    pub window: Window,
    pub exe: String,
    pub path: String,
    pub title: String,
    pub monitor: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Window {
    pub(crate) handle: usize,
    pub(crate) process_id: u32,
}

pub struct CapturableWindow {
    pub window: Window,
    pub title: String,
    pub fullscreen: bool,
}

pub fn capture_windows() -> Vec<CapturableWindow> {
    unsafe extern "system" fn add(hwnd: HWND, context: LPARAM) -> windows::core::BOOL {
        unsafe {
            let mut pid = 0;
            if !IsWindowVisible(hwnd).as_bool()
                || IsIconic(hwnd).as_bool()
                || GetWindowThreadProcessId(hwnd, Some(&mut pid)) == 0
                || pid == std::process::id()
            {
                return TRUE;
            }
            let mut title = [0u16; 256];
            let length = GetWindowTextW(hwnd, &mut title).max(0) as usize;
            let title = String::from_utf16_lossy(&title[..length]);
            if title.trim().is_empty() {
                return TRUE;
            }
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let mut bounds = RECT::default();
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
            let fullscreen = GetWindowRect(hwnd, &mut bounds).is_ok()
                && GetMonitorInfoW(monitor, &mut info).as_bool()
                && bounds.left <= info.rcMonitor.left
                && bounds.top <= info.rcMonitor.top
                && bounds.right >= info.rcMonitor.right
                && bounds.bottom >= info.rcMonitor.bottom;
            let choices = &mut *(context.0 as *mut Vec<CapturableWindow>);
            choices.push(CapturableWindow {
                window: Window {
                    handle: hwnd.0 as usize,
                    process_id: pid,
                },
                title,
                fullscreen,
            });
        }
        TRUE
    }

    let mut choices = Vec::new();
    // EnumWindows calls add synchronously on this thread.
    let _ = unsafe { EnumWindows(Some(add), LPARAM((&raw mut choices) as isize)) };
    choices
}

pub fn current() -> Option<Foreground> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let path = process_path(pid)?;

        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut buf).max(0) as usize;
        Some(Foreground {
            window: Window {
                handle: hwnd.0 as usize,
                process_id: pid,
            },
            exe: file_name(&path).to_owned(),
            path,
            title: String::from_utf16_lossy(&buf[..len]),
            monitor: MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL).0 as usize,
        })
    }
}

fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        if !ok {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..size as usize]))
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::file_name;

    #[test]
    fn extracts_file_name_from_windows_path() {
        assert_eq!(file_name(r"C:\Games\CS2\cs2.exe"), "cs2.exe");
        assert_eq!(file_name("cs2.exe"), "cs2.exe");
        assert_eq!(file_name(r"D:\a\b\"), "");
    }
}
