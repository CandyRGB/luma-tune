use std::mem;

use windows::core::{w, BOOL, Interface, PCWSTR};
use windows::Win32::Foundation::{LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, IDXGIOutput6};
use windows::Win32::Graphics::Gdi::{
    CreateDCW, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

#[derive(Clone)]
pub struct Display {
    pub device: String,
    pub adapter: String,
    pub width: i32,
    pub height: i32,
    pub primary: bool,
    pub color_space: Option<i32>,
    pub bits_per_color: Option<u32>,
}

impl Display {
    pub fn hdr(&self) -> bool {
        self.color_space.is_some_and(|c| c != 0)
    }

    pub fn looks_virtual(&self) -> bool {
        let a = self.adapter.to_lowercase();
        a.contains("virtual") || a.contains("indirect") || a.contains("basic")
    }

    pub fn hdc(&self) -> HDC {
        let name = wide(&self.device);
        unsafe { CreateDCW(w!(""), PCWSTR(name.as_ptr()), w!(""), None) }
    }
}

pub fn set_dpi_awareness() {
    unsafe {
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }
}

pub fn enumerate() -> Vec<Display> {
    let mut raw: Vec<(HMONITOR, MONITORINFOEXW)> = Vec::new();
    unsafe {
        let ptr = &mut raw as *mut Vec<(HMONITOR, MONITORINFOEXW)> as isize;
        let _ = EnumDisplayMonitors(None, None, Some(collect), LPARAM(ptr));
    }
    let spaces = outputs();
    raw.iter()
        .map(|(h, mi)| {
            let found = spaces.iter().find(|s| s.0 == *h);
            Display {
                device: wide_to_string(&mi.szDevice),
                adapter: found.map(|s| s.3.clone()).unwrap_or_default(),
                width: mi.monitorInfo.rcMonitor.right - mi.monitorInfo.rcMonitor.left,
                height: mi.monitorInfo.rcMonitor.bottom - mi.monitorInfo.rcMonitor.top,
                primary: mi.monitorInfo.dwFlags & 1 != 0,
                color_space: found.map(|s| s.1),
                bits_per_color: found.map(|s| s.2),
            }
        })
        .collect()
}

pub fn primary(displays: &[Display]) -> Option<&Display> {
    displays
        .iter()
        .find(|d| d.primary)
        .or_else(|| displays.first())
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_to_string(w: &[u16]) -> String {
    let end = w.iter().position(|c| *c == 0).unwrap_or(w.len());
    String::from_utf16_lossy(&w[..end])
}

unsafe extern "system" fn collect(
    hmon: HMONITOR,
    _hdc: HDC,
    _rc: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let out = unsafe { &mut *(data.0 as *mut Vec<(HMONITOR, MONITORINFOEXW)>) };
    let mut mi: MONITORINFOEXW = unsafe { mem::zeroed() };
    mi.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
    if unsafe { GetMonitorInfoW(hmon, &mut mi.monitorInfo) }.as_bool() {
        out.push((hmon, mi));
    }
    TRUE
}

fn outputs() -> Vec<(HMONITOR, i32, u32, String)> {
    let mut out = Vec::new();
    unsafe {
        let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
            Ok(f) => f,
            Err(_) => return out,
        };
        let mut i = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(i) {
            let name = adapter
                .GetDesc1()
                .map(|d| wide_to_string(&d.Description))
                .unwrap_or_default();
            let mut j = 0u32;
            while let Ok(output) = adapter.EnumOutputs(j) {
                if let Ok(desc) = output.cast::<IDXGIOutput6>().and_then(|o| o.GetDesc1()) {
                    out.push((
                        desc.Monitor,
                        desc.ColorSpace.0,
                        desc.BitsPerColor,
                        name.clone(),
                    ));
                }
                j += 1;
            }
            i += 1;
        }
    }
    out
}
