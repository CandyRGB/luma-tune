use lumatune_core::Ramp;
use windows::Win32::Graphics::Gdi::{DeleteDC, HDC};
use windows::Win32::UI::ColorSystem::{GetDeviceGammaRamp, SetDeviceGammaRamp};

use crate::display::Display;

pub fn read(hdc: HDC) -> Option<Ramp> {
    let mut r: Ramp = [0; 768];
    let ok = unsafe { GetDeviceGammaRamp(hdc, r.as_mut_ptr() as *mut core::ffi::c_void) };
    ok.as_bool().then_some(r)
}

pub fn write(hdc: HDC, r: &Ramp) -> bool {
    unsafe { SetDeviceGammaRamp(hdc, r.as_ptr() as *const core::ffi::c_void) }.as_bool()
}

pub(crate) fn write_verified(hdc: HDC, expected: &Ramp) -> bool {
    // The driver may quantize a 16-bit entry to eight bits or silently reject the write.
    write(hdc, expected)
        && read(hdc).is_some_and(|actual| {
            actual
                .iter()
                .zip(expected)
                .all(|(a, b)| a.abs_diff(*b) <= 257)
        })
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

pub struct Session {
    hdc: HDC,
    original: Ramp,
    device: String,
    recovery_key: String,
    active: bool,
}

impl Session {
    pub fn open(display: &Display) -> Option<Self> {
        let recovery_key = display.identity.clone()?;
        if crate::state::has_pending(display) {
            return None;
        }
        let hdc = display.hdc();
        if hdc.0.is_null() {
            return None;
        }
        let Some(original) = read(hdc) else {
            unsafe {
                let _ = DeleteDC(hdc);
            }
            return None;
        };
        Some(Self {
            hdc,
            original,
            device: display.device.clone(),
            recovery_key,
            active: false,
        })
    }

    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn original(&self) -> &Ramp {
        &self.original
    }

    pub fn apply(&mut self, ramp: &Ramp) -> bool {
        if !self.active {
            if !crate::state::mark_active(&self.recovery_key, &self.original) {
                return false;
            }
            self.active = true;
        }
        write_verified(self.hdc, ramp)
    }

    pub fn restore(&mut self) -> bool {
        if self.active {
            if !write_verified(self.hdc, &self.original) {
                return false;
            }
            crate::state::clear_active(&self.recovery_key);
            self.active = false;
        }
        true
    }

    pub fn detach(mut self) {
        self.active = false;
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.restore();
        unsafe {
            let _ = DeleteDC(self.hdc);
        }
    }
}
