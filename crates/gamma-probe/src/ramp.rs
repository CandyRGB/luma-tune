use std::fs;
use std::path::PathBuf;

use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::UI::ColorSystem::{GetDeviceGammaRamp, SetDeviceGammaRamp};

pub type Ramp = [u16; 768];

pub fn read(hdc: HDC) -> Option<Ramp> {
    let mut r: Ramp = [0; 768];
    let ok = unsafe { GetDeviceGammaRamp(hdc, r.as_mut_ptr() as *mut core::ffi::c_void) };
    ok.as_bool().then_some(r)
}

pub fn write(hdc: HDC, r: &Ramp) -> bool {
    unsafe { SetDeviceGammaRamp(hdc, r.as_ptr() as *const core::ffi::c_void) }.as_bool()
}

pub fn identity() -> Ramp {
    let mut r: Ramp = [0; 768];
    for i in 0..256 {
        let v = (i * 257) as u16;
        r[i] = v;
        r[256 + i] = v;
        r[512 + i] = v;
    }
    r
}

pub fn is_identity(r: &Ramp) -> bool {
    *r == identity()
}

pub fn probe_pattern() -> Ramp {
    let mut r: Ramp = [0; 768];
    for i in 0..256 {
        let v = if i == 255 {
            65535u16
        } else {
            (i * 257 + 3) as u16
        };
        r[i] = v;
        r[256 + i] = v;
        r[512 + i] = v;
    }
    r
}

pub fn brighten(gamma: f32) -> Ramp {
    let mut r: Ramp = [0; 768];
    for i in 0..256 {
        let x = i as f32 / 255.0;
        let v = (x.powf(1.0 / gamma) * 65535.0).round() as u16;
        r[i] = v;
        r[256 + i] = v;
        r[512 + i] = v;
    }
    r
}

pub fn max_delta(a: &Ramp, b: &Ramp) -> u16 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

pub fn exact_matches(a: &Ramp, b: &Ramp) -> usize {
    a.iter().zip(b.iter()).filter(|(x, y)| x == y).count()
}

pub fn eight_bit_only(r: &Ramp) -> bool {
    r.iter().all(|v| (*v as u32) % 257 == 0)
}

pub fn backup_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("LumaTune")
        .join("probe")
}

fn sanitize(device: &str) -> String {
    device
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

pub fn save_backup(device: &str, r: &Ramp) -> Option<PathBuf> {
    let dir = backup_dir();
    fs::create_dir_all(&dir).ok()?;
    let path = dir.join(format!("{}.ramp", sanitize(device)));
    let mut bytes = Vec::with_capacity(1536);
    for v in r.iter() {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    fs::write(&path, bytes).ok()?;
    Some(path)
}

pub fn load_backup(device: &str) -> Option<(PathBuf, Ramp)> {
    let path = backup_dir().join(format!("{}.ramp", sanitize(device)));
    let bytes = fs::read(&path).ok()?;
    if bytes.len() != 1536 {
        return None;
    }
    let mut r: Ramp = [0; 768];
    for (i, chunk) in bytes.chunks_exact(2).enumerate() {
        r[i] = u16::from_le_bytes([chunk[0], chunk[1]]);
    }
    Some((path, r))
}
