use std::ffi::OsString;
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, GetOpenFileNameW, OFN_DONTADDTORECENT, OFN_FILEMUSTEXIST,
    OFN_NOCHANGEDIR, OPENFILENAMEW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
use windows::core::{PCWSTR, PWSTR, w};

pub fn cover() -> io::Result<Option<PathBuf>> {
    open(
        w!("选择预设主图"),
        "图片文件\0*.png;*.jpg;*.jpeg;*.webp;*.bmp\0\0",
    )
}

pub fn executable() -> io::Result<Option<PathBuf>> {
    open(w!("选择应用程序"), "应用程序\0*.exe\0\0")
}

fn open(title: PCWSTR, filter: &str) -> io::Result<Option<PathBuf>> {
    let mut path = [0u16; 4096];
    let filter: Vec<u16> = filter.encode_utf16().collect();
    let mut dialog = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        // SAFETY: the picker is called from the Slint window's UI thread.
        hwndOwner: unsafe { GetActiveWindow() },
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(path.as_mut_ptr()),
        nMaxFile: path.len() as u32,
        lpstrTitle: title,
        Flags: OFN_DONTADDTORECENT | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR,
        ..Default::default()
    };
    // SAFETY: both buffers outlive the synchronous dialog call.
    if unsafe { GetOpenFileNameW(&mut dialog) }.as_bool() {
        let end = path
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(path.len());
        return Ok(Some(PathBuf::from(OsString::from_wide(&path[..end]))));
    }
    // SAFETY: this call retrieves the error from the dialog on this thread.
    let error = unsafe { CommDlgExtendedError() }.0;
    if error == 0 {
        return Ok(None);
    }
    Err(io::Error::other(format!("文件选择窗口错误 0x{error:04X}")))
}
