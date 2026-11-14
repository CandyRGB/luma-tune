use std::fs;
use std::path::PathBuf;

use crate::display::Display;

fn dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("LumaTune")
}

fn flag() -> PathBuf {
    dir().join("active.flag")
}

pub fn mark_active(device: &str) {
    let _ = fs::create_dir_all(dir());
    let _ = fs::write(flag(), device);
}

pub fn clear_active() {
    let _ = fs::remove_file(flag());
}

pub fn pending_device() -> Option<String> {
    fs::read_to_string(flag()).ok().map(|s| s.trim().to_string())
}

pub fn heal(displays: &[Display]) -> bool {
    let Some(stale) = pending_device() else {
        return false;
    };
    let target = displays
        .iter()
        .find(|d| d.device == stale)
        .or_else(|| crate::display::primary(displays));
    if let Some(d) = target {
        let hdc = d.hdc();
        if !hdc.0.is_null() {
            let _ = crate::ramp::write(hdc, &crate::ramp::identity());
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::DeleteDC(hdc);
            }
        }
    }
    clear_active();
    true
}
