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
    active: bool,
}

impl Session {
    pub fn open(display: &Display) -> Option<Self> {
        let hdc = display.hdc();
        if hdc.0.is_null() {
            return None;
        }
        let original = read(hdc)?;
        Some(Self {
            hdc,
            original,
            device: display.device.clone(),
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
            crate::state::mark_active(&self.device);
            self.active = true;
        }
        write(self.hdc, ramp)
    }

    pub fn restore(&mut self) {
        if self.active {
            write(self.hdc, &self.original);
            crate::state::clear_active();
            self.active = false;
        }
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
