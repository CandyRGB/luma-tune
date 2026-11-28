use std::collections::HashMap;
use std::mem;

use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, DisplayConfigGetDeviceInfo,
    GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig,
};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS};
use windows::Win32::Foundation::{LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, IDXGIOutput6};
use windows::Win32::Graphics::Gdi::{
    CreateDCW, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
use windows::core::{BOOL, Interface, PCWSTR, w};

#[derive(Clone, PartialEq, Eq)]
pub struct Display {
    pub device: String,
    pub handle: usize,
    pub identity: Option<String>,
    pub name: String,
    pub adapter: String,
    pub width: i32,
    pub height: i32,
    pub primary: bool,
    pub color_space: Option<i32>,
    pub bits_per_color: Option<u32>,
}

impl Display {
    pub fn key(&self) -> &str {
        self.identity.as_deref().unwrap_or(&self.device)
    }

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
    let mut names = display_targets();
    let mut displays: Vec<_> = raw
        .iter()
        .map(|(h, mi)| {
            let found = spaces.iter().find(|s| s.0 == *h);
            let device = wide_to_string(&mi.szDevice);
            let mut targets = names
                .remove(&device.to_ascii_lowercase())
                .unwrap_or_default();
            targets.sort_unstable();
            targets.dedup();
            let identity = (!targets.is_empty() && targets.iter().all(|(id, _)| !id.is_empty()))
                .then(|| {
                    targets
                        .iter()
                        .map(|(id, _)| id.as_str())
                        .collect::<Vec<_>>()
                        .join("|")
                });
            let mut label = targets
                .into_iter()
                .map(|(_, name)| name)
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>()
                .join(" / ");
            if label.is_empty() {
                label = device
                    .strip_prefix(r"\\.\DISPLAY")
                    .map(|number| format!("显示器 {number}"))
                    .unwrap_or_else(|| device.clone());
            }
            Display {
                device,
                handle: h.0 as usize,
                identity,
                name: label,
                adapter: found.map(|s| s.3.clone()).unwrap_or_default(),
                width: mi.monitorInfo.rcMonitor.right - mi.monitorInfo.rcMonitor.left,
                height: mi.monitorInfo.rcMonitor.bottom - mi.monitorInfo.rcMonitor.top,
                primary: mi.monitorInfo.dwFlags & 1 != 0,
                color_space: found.map(|s| s.1),
                bits_per_color: found.map(|s| s.2),
            }
        })
        .collect();
    displays.sort_by(|a, b| b.primary.cmp(&a.primary).then(a.device.cmp(&b.device)));
    displays
}

fn display_targets() -> HashMap<String, Vec<(String, String)>> {
    let mut targets = HashMap::<String, Vec<(String, String)>>::new();
    unsafe {
        for _ in 0..3 {
            let (mut path_count, mut mode_count) = (0, 0);
            if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
                != ERROR_SUCCESS
            {
                break;
            }
            let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
            let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
            let result = QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            );
            if result == ERROR_INSUFFICIENT_BUFFER {
                continue;
            }
            if result != ERROR_SUCCESS {
                break;
            }
            for path in paths.iter().take(path_count as usize) {
                let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
                    header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                        r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                        size: size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                        adapterId: path.sourceInfo.adapterId,
                        id: path.sourceInfo.id,
                    },
                    ..Default::default()
                };
                if DisplayConfigGetDeviceInfo(std::ptr::addr_of_mut!(source).cast()) != 0 {
                    continue;
                }
                let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME {
                    header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                        r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                        size: size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                        adapterId: path.targetInfo.adapterId,
                        id: path.targetInfo.id,
                    },
                    ..Default::default()
                };
                let _ = DisplayConfigGetDeviceInfo(std::ptr::addr_of_mut!(target).cast());
                targets
                    .entry(wide_to_string(&source.viewGdiDeviceName).to_ascii_lowercase())
                    .or_default()
                    .push((
                        wide_to_string(&target.monitorDevicePath).to_ascii_lowercase(),
                        wide_to_string(&target.monitorFriendlyDeviceName),
                    ));
            }
            break;
        }
    }
    targets
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

unsafe extern "system" fn collect(hmon: HMONITOR, _hdc: HDC, _rc: *mut RECT, data: LPARAM) -> BOOL {
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
