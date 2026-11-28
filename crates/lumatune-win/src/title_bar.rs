use std::ffi::c_void;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR, DwmDefWindowProc, DwmExtendFrameIntoClientArea,
    DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, HTCAPTION, HTCLIENT, HTTOP, HTTOPLEFT, HTTOPRIGHT, IsZoomed, NCCALCSIZE_PARAMS,
    SM_CXPADDEDBORDER, SM_CYSIZEFRAME, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowPos, WM_ACTIVATE, WM_DPICHANGED, WM_NCCALCSIZE, WM_NCDESTROY,
    WM_NCHITTEST, WM_NCMOUSELEAVE,
};

const SUBCLASS_ID: usize = 1;
pub const HEIGHT: i32 = 40;
// Covers the two Slint buttons at x=4..40 and x=44..80.
const NAV_BUTTONS_WIDTH: i32 = 84;

/// Extends an existing overlapped window into its caption while keeping the
/// OS caption buttons, resizing border and system menu.
pub fn integrate(window: usize) -> bool {
    let hwnd = HWND(window as *mut c_void);
    // The caller supplies the UI thread's live HWND. The subclass stores no pointers.
    unsafe {
        if !SetWindowSubclass(hwnd, Some(caption_proc), SUBCLASS_ID, 0).as_bool() {
            return false;
        }
        if extend_frame(hwnd).is_ok()
            && SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER,
            )
            .is_ok()
        {
            return true;
        }
        let _ = RemoveWindowSubclass(hwnd, Some(caption_proc), SUBCLASS_ID);
        let _ = DwmExtendFrameIntoClientArea(hwnd, &MARGINS::default());
        false
    }
}

fn scale(length: i32, dpi: u32) -> i32 {
    (length * dpi as i32 + 48) / 96
}

fn resize_border(dpi: u32) -> i32 {
    unsafe {
        GetSystemMetricsForDpi(SM_CYSIZEFRAME, dpi) + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi)
    }
}

/// Logical height clipped by the monitor edge when the HWND is maximized.
pub fn maximized_inset(window: usize) -> f32 {
    let dpi = unsafe { GetDpiForWindow(HWND(window as *mut c_void)) }.max(96);
    resize_border(dpi) as f32 * 96.0 / dpi as f32
}

pub fn set_colors(window: usize, background: u32, foreground: u32) {
    let hwnd = HWND(window as *mut c_void);
    for (attribute, color) in [
        (DWMWA_CAPTION_COLOR, background),
        (DWMWA_TEXT_COLOR, foreground),
    ] {
        // Windows versions without caption color support keep the system colors.
        let _ = unsafe { DwmSetWindowAttribute(hwnd, attribute, (&raw const color).cast(), 4) };
    }
}

unsafe fn extend_frame(hwnd: HWND) -> windows::core::Result<()> {
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    unsafe {
        DwmExtendFrameIntoClientArea(
            hwnd,
            &MARGINS {
                cyTopHeight: scale(HEIGHT, dpi) + resize_border(dpi),
                ..MARGINS::default()
            },
        )
    }
}

unsafe extern "system" fn caption_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    _data: usize,
) -> LRESULT {
    // These messages and their pointer arguments come from the window manager.
    unsafe {
        match message {
            WM_NCCALCSIZE => {
                let client = if wparam.0 != 0 {
                    &raw mut (*(lparam.0 as *mut NCCALCSIZE_PARAMS)).rgrc[0]
                } else {
                    lparam.0 as *mut RECT
                };
                let window_top = (*client).top;
                DefSubclassProc(hwnd, message, wparam, lparam);
                // Preserve the default frame and its maximized work-area adjustment.
                // Only reclaim the height used by the standard caption.
                // DWM requires a zero top inset for native caption button hit-testing.
                (*client).top = window_top;
                return LRESULT(0);
            }
            WM_NCHITTEST | WM_NCMOUSELEAVE => {
                let mut result = LRESULT(0);
                if DwmDefWindowProc(hwnd, message, wparam, lparam, &mut result).as_bool() {
                    return result;
                }
                if message == WM_NCHITTEST {
                    let hit = DefSubclassProc(hwnd, message, wparam, lparam);
                    if hit.0 == HTCLIENT as isize || hit.0 == HTCAPTION as isize {
                        let mut point = POINT {
                            x: lparam.0 as i16 as i32,
                            y: (lparam.0 >> 16) as i16 as i32,
                        };
                        let dpi = GetDpiForWindow(hwnd);
                        let mut client = RECT::default();
                        if ScreenToClient(hwnd, &mut point).as_bool()
                            && GetClientRect(hwnd, &mut client).is_ok()
                            && point.x >= 0
                            && point.x < client.right
                            && point.y >= 0
                        {
                            let maximized = IsZoomed(hwnd).as_bool();
                            if point.y < scale(4, dpi) && !maximized {
                                let grip = scale(8, dpi);
                                return LRESULT(if point.x < grip {
                                    HTTOPLEFT as isize
                                } else if point.x >= client.right - grip {
                                    HTTOPRIGHT as isize
                                } else {
                                    HTTOP as isize
                                });
                            }
                            let caption_bottom =
                                scale(HEIGHT, dpi) + if maximized { resize_border(dpi) } else { 0 };
                            return LRESULT(
                                if point.y < caption_bottom
                                    && point.x >= scale(NAV_BUTTONS_WIDTH, dpi)
                                {
                                    HTCAPTION as isize
                                } else {
                                    HTCLIENT as isize
                                },
                            );
                        }
                    }
                    return hit;
                }
            }
            WM_ACTIVATE => {
                let _ = extend_frame(hwnd);
            }
            WM_DPICHANGED => {
                let result = DefSubclassProc(hwnd, message, wparam, lparam);
                let _ = extend_frame(hwnd);
                return result;
            }
            WM_NCDESTROY => {
                let _ = RemoveWindowSubclass(hwnd, Some(caption_proc), id);
            }
            _ => {}
        }
        DefSubclassProc(hwnd, message, wparam, lparam)
    }
}
