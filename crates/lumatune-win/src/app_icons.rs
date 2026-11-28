use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppRGBA, IWICImagingFactory, IWICPalette,
    WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize,
};
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON};
use windows::core::PCWSTR;

pub struct AppIcon {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

struct IconHandle(HICON);

impl Drop for IconHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: this handle is owned by the caller of ExtractIconExW.
            let _ = unsafe { DestroyIcon(self.0) };
        }
    }
}

struct ComApartment(bool);

impl ComApartment {
    fn enter() -> Option<Self> {
        // SAFETY: initialization and destruction run on the same thread.
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if result == RPC_E_CHANGED_MODE {
            // The caller already initialized COM using another apartment model.
            Some(Self(false))
        } else {
            result.ok().ok().map(|()| Self(true))
        }
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: balances this guard's successful CoInitializeEx, including S_FALSE.
            unsafe { CoUninitialize() };
        }
    }
}

fn extract(source: &Path) -> Option<AppIcon> {
    let path: Vec<u16> = source.as_os_str().encode_wide().chain([0]).collect();
    let mut icon = IconHandle(HICON::default());
    // SAFETY: path is a terminated UTF-16 string and icon has space for one returned handle.
    if unsafe { ExtractIconExW(PCWSTR(path.as_ptr()), 0, Some(&mut icon.0), None, 1) } != 1
        || icon.0.is_invalid()
    {
        return None;
    }

    let _com = ComApartment::enter()?;
    let mut width = 0;
    let mut height = 0;
    // SAFETY: COM is initialized, icon stays alive while WIC reads it, and CopyPixels
    // receives a buffer sized from the bitmap dimensions and the RGBA stride.
    let rgba = unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()?;
        let bitmap = factory.CreateBitmapFromHICON(icon.0).ok()?;
        bitmap.GetSize(&mut width, &mut height).ok()?;
        if !(1..=256).contains(&width) || !(1..=256).contains(&height) {
            return None;
        }
        let converter = factory.CreateFormatConverter().ok()?;
        converter
            .Initialize(
                &bitmap,
                &GUID_WICPixelFormat32bppRGBA,
                WICBitmapDitherTypeNone,
                None::<&IWICPalette>,
                0.0,
                WICBitmapPaletteTypeCustom,
            )
            .ok()?;
        let mut pixels = vec![0; (width * height * 4) as usize];
        converter
            .CopyPixels(std::ptr::null(), width * 4, &mut pixels)
            .ok()?;
        pixels
    };
    rgba.chunks_exact(4)
        .any(|pixel| pixel[3] != 0)
        .then_some(AppIcon {
            width,
            height,
            rgba,
        })
}

pub fn cache_path(exe: &str) -> Option<PathBuf> {
    let name = exe.to_ascii_lowercase();
    if name.len() <= 4
        || !name.ends_with(".exe")
        || name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
        || name.chars().any(char::is_control)
    {
        return None;
    }
    Some(
        crate::settings::path()
            .parent()?
            .join("app-icons")
            .join(format!("app-{name}.png")),
    )
}

pub fn capture(exe: &str, source: &Path) -> Option<AppIcon> {
    let icon = extract(source)?;
    if let Some(path) = cache_path(exe) {
        let _ = save(&path, &icon);
    }
    Some(icon)
}

fn save(path: &Path, icon: &AppIcon) -> std::io::Result<()> {
    let parent = path.parent().expect("app icon cache path has a parent");
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    let mut encoder = png::Encoder::new(file.as_file_mut(), icon.width, icon.height);
    encoder.set_color(png::ColorType::Rgba);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&icon.rgba)?;
    writer.finish()?;
    file.persist(path)?;
    Ok(())
}
