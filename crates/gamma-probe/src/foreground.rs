use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND, RECT};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Shell::{
    QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GWL_STYLE, GetClassNameW, GetForegroundWindow, GetWindowLongW, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, WS_CAPTION, WS_POPUP, WS_THICKFRAME,
};

pub struct Snapshot {
    pub title: String,
    pub class: String,
    pub process: String,
    pub pid: u32,
    pub width: i32,
    pub height: i32,
    pub left: i32,
    pub top: i32,
    pub popup: bool,
    pub caption: bool,
    pub thick_frame: bool,
    pub layered: bool,
    pub fullscreen_state: i32,
}

pub fn capture() -> Option<Snapshot> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf).max(0) as usize;
        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf).max(0) as usize;

        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let exstyle = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let state = SHQueryUserNotificationState()
            .map(|s| s.0)
            .unwrap_or(-1);

        Some(Snapshot {
            title: String::from_utf16_lossy(&title_buf[..title_len]),
            class: String::from_utf16_lossy(&class_buf[..class_len]),
            process: process_path(pid),
            pid,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
            left: rect.left,
            top: rect.top,
            popup: style & WS_POPUP.0 != 0,
            caption: style & WS_CAPTION.0 != 0,
            thick_frame: style & WS_THICKFRAME.0 != 0,
            layered: exstyle & 0x0008_0000 != 0,
            fullscreen_state: state,
        })
    }
}

pub fn state_label(state: i32) -> &'static str {
    if state == QUNS_RUNNING_D3D_FULL_SCREEN.0 {
        "独占全屏（系统报告 D3D 全屏）"
    } else if state == QUNS_BUSY.0 {
        "全屏应用（系统标记为忙碌）"
    } else if state < 0 {
        "查询失败"
    } else {
        "非全屏"
    }
}

pub fn verdict(s: &Snapshot, screen_w: i32, screen_h: i32) -> String {
    if s.fullscreen_state == QUNS_RUNNING_D3D_FULL_SCREEN.0 {
        return "独占全屏".into();
    }
    if s.fullscreen_state == QUNS_BUSY.0 {
        return if s.width >= screen_w && s.height >= screen_h && s.popup && !s.caption {
            "无边框全屏".into()
        } else {
            "全屏应用（系统标记忙碌，窗口形态不像全屏）".into()
        };
    }
    if s.class == "Progman" || s.class == "WorkerW" {
        return "桌面".into();
    }
    if s.width >= screen_w && s.height >= screen_h && s.popup && !s.caption {
        return "窗口化（恰好铺满屏幕，但系统未标记为全屏）".into();
    }
    "窗口化".into()
}

fn process_path(pid: u32) -> String {
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return String::new();
        };
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
        if ok {
            String::from_utf16_lossy(&buf[..size as usize])
        } else {
            String::new()
        }
    }
}
