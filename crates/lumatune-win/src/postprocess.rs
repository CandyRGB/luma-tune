use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};

use lumatune_core::{PostProcessParams, Ramp};

use crate::foreground::Window;

#[derive(Clone, Copy, PartialEq)]
pub struct Target {
    window: Window,
    params: PostProcessParams,
}

impl Target {
    pub fn new(window: Window, params: PostProcessParams) -> Option<Self> {
        let params = params.normalized();
        (window.process_id != std::process::id() && !params.is_neutral())
            .then_some(Self { window, params })
    }
}

struct Worker {
    target: Target,
    stop: Arc<AtomicBool>,
    task: JoinHandle<i32>,
}

#[derive(Default)]
pub struct Processor {
    worker: Option<Worker>,
    failed: Option<Target>,
}

impl Processor {
    pub fn update(&mut self, target: Option<Target>) -> Result<(), String> {
        if self.failed != target {
            self.failed = None;
        }
        if let Some(worker) = &self.worker {
            if Some(worker.target) != target {
                worker.stop.store(true, Ordering::Release);
            }
            if !worker.task.is_finished() {
                return Ok(());
            }
        }
        if let Some(worker) = self.worker.take() {
            let result = worker.task.join().unwrap_or(i32::MIN);
            if result < 0 && Some(worker.target) == target && !worker.stop.load(Ordering::Acquire) {
                self.failed = target;
                return Err(error_message(result));
            }
        }
        if let Some(target) = target.filter(|target| Some(*target) != self.failed) {
            let stop = Arc::new(AtomicBool::new(false));
            let token = Arc::clone(&stop);
            match thread::Builder::new()
                .name("lumatune-capture".into())
                .spawn(move || run(target, token))
            {
                Ok(task) => self.worker = Some(Worker { target, stop, task }),
                Err(error) => {
                    self.failed = Some(target);
                    return Err(error.to_string());
                }
            }
        }
        Ok(())
    }
}

impl Drop for Processor {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.stop.store(true, Ordering::Release);
            let _ = worker.task.join();
        }
    }
}

fn error_message(code: i32) -> String {
    match code as u32 {
        0x80004001 => "当前环境不支持 Windows 窗口捕获后处理。".into(),
        0x80070005 => "Windows 拒绝捕获此应用窗口。".into(),
        0x800705B4 => "未收到应用画面，请检查游戏是否使用无边框窗口模式。".into(),
        0x80070032 => "画面后处理需要 SDR 显示器和无边框窗口模式。".into(),
        _ => format!("启动或渲染应用画面失败（0x{:08X}）。", code as u32),
    }
}

#[cfg(all(target_env = "msvc", target_os = "windows"))]
fn run(target: Target, stop: Arc<AtomicBool>) -> i32 {
    use std::ffi::c_void;

    unsafe extern "C" {
        fn lumatune_postprocess_run(
            window: usize,
            process_id: u32,
            tone: f32,
            bloom: f32,
            saturation: f32,
            clarity: f32,
            vibrance: f32,
            context: *const c_void,
            cancelled: extern "C" fn(*const c_void) -> bool,
        ) -> i32;
    }

    extern "C" fn cancelled(context: *const c_void) -> bool {
        // The worker's Arc keeps the token alive for the entire synchronous call.
        unsafe { &*context.cast::<AtomicBool>() }.load(Ordering::Acquire)
    }

    unsafe {
        lumatune_postprocess_run(
            target.window.handle,
            target.window.process_id,
            target.params.hdr_toning,
            target.params.bloom,
            target.params.saturation,
            target.params.clarity,
            target.params.vibrance,
            Arc::as_ptr(&stop).cast(),
            cancelled,
        )
    }
}

#[cfg(not(all(target_env = "msvc", target_os = "windows")))]
fn run(_: Target, _: Arc<AtomicBool>) -> i32 {
    windows::Win32::Foundation::E_NOTIMPL.0
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CaptureSource {
    Monitor(usize),
    Window(Window),
}

pub struct PreviewFrame {
    pub width: u32,
    pub height: u32,
    pub before: Vec<u8>,
    pub after: Vec<u8>,
}

struct PreviewState {
    revision: u64,
    effects: PostProcessParams,
    ramp: Option<Ramp>,
    frame: Option<PreviewFrame>,
}

pub struct Preview {
    stop: Arc<AtomicBool>,
    state: Arc<Mutex<PreviewState>>,
    task: Option<JoinHandle<i32>>,
}

impl Preview {
    pub fn start(source: CaptureSource, editor: usize) -> std::io::Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let state = Arc::new(Mutex::new(PreviewState {
            revision: 0,
            effects: PostProcessParams::default(),
            ramp: None,
            frame: None,
        }));
        let token = Arc::clone(&stop);
        let frames = Arc::clone(&state);
        let task = thread::Builder::new()
            .name("lumatune-preview".into())
            .spawn(move || run_preview(source, editor, &token, &frames))?;
        Ok(Self {
            stop,
            state,
            task: Some(task),
        })
    }

    pub fn update(&self, effects: PostProcessParams, ramp: Option<Ramp>) {
        let mut state = self.state.lock().unwrap();
        if state.effects != effects || state.ramp != ramp {
            state.effects = effects;
            state.ramp = ramp;
            state.revision = state.revision.wrapping_add(1);
        }
    }

    pub fn frame(&self) -> Option<PreviewFrame> {
        self.state.lock().unwrap().frame.take()
    }

    pub fn finished(&mut self) -> Option<String> {
        if self.task.as_ref().is_some_and(JoinHandle::is_finished) {
            let code = self.task.take()?.join().unwrap_or(i32::MIN);
            Some(match code as u32 {
                0x80070578 => "捕获源已关闭或最小化，请重新选择。".into(),
                0x800705B4 => "未收到画面，请确认应用正在显示。".into(),
                0x80070032 => "无法预览所选来源。".into(),
                _ => error_message(code),
            })
        } else {
            None
        }
    }
}

impl Drop for Preview {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(task) = self.task.take() {
            let _ = task.join();
        }
    }
}

#[cfg(all(target_env = "msvc", target_os = "windows"))]
fn run_preview(
    source: CaptureSource,
    editor: usize,
    stop: &AtomicBool,
    state: &Mutex<PreviewState>,
) -> i32 {
    use std::ffi::c_void;

    struct Context<'a> {
        stop: &'a AtomicBool,
        state: &'a Mutex<PreviewState>,
    }

    unsafe extern "C" {
        fn lumatune_preview_run(
            window: usize,
            process_id: u32,
            monitor: usize,
            editor: usize,
            context: *const c_void,
            cancelled: extern "C" fn(*const c_void) -> bool,
            settings: extern "C" fn(*const c_void, *mut [f32; 5]) -> u64,
            pixels: extern "C" fn(*const c_void, u64, u32, u32, *const u8, u32, *const u8, u32),
        ) -> i32;
    }

    extern "C" fn cancelled(context: *const c_void) -> bool {
        // The synchronous native call borrows Context for the whole session.
        unsafe { &*context.cast::<Context<'_>>() }
            .stop
            .load(Ordering::Acquire)
    }

    extern "C" fn settings(context: *const c_void, output: *mut [f32; 5]) -> u64 {
        let state = unsafe { &*context.cast::<Context<'_>>() }
            .state
            .lock()
            .unwrap();
        let effects = state.effects;
        unsafe {
            output.write([
                effects.hdr_toning,
                effects.bloom,
                effects.saturation,
                effects.clarity,
                effects.vibrance,
            ]);
        }
        state.revision
    }

    extern "C" fn pixels(
        context: *const c_void,
        revision: u64,
        width: u32,
        height: u32,
        before: *const u8,
        before_stride: u32,
        after: *const u8,
        after_stride: u32,
    ) {
        let context = unsafe { &*context.cast::<Context<'_>>() };
        let ramp = {
            let state = context.state.lock().unwrap();
            if state.revision != revision {
                return;
            }
            state.ramp
        };
        let copy = |source: *const u8, stride: u32, ramp: Option<&Ramp>| {
            let row_len = width as usize * 4;
            let mut rgba = Vec::with_capacity(row_len * height as usize);
            for y in 0..height as usize {
                // Native staging textures are mapped for the duration of this callback.
                let row =
                    unsafe { std::slice::from_raw_parts(source.add(y * stride as usize), row_len) };
                for pixel in row.chunks_exact(4) {
                    let [r, g, b] = [pixel[2], pixel[1], pixel[0]];
                    let rgb = ramp.map_or([r, g, b], |lut| {
                        std::array::from_fn(|channel| {
                            let value = [r, g, b][channel];
                            ((u32::from(lut[channel * 256 + value as usize]) + 128) / 257) as u8
                        })
                    });
                    rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
                }
            }
            rgba
        };
        let frame = PreviewFrame {
            width,
            height,
            before: copy(before, before_stride, None),
            after: copy(after, after_stride, ramp.as_ref()),
        };
        let mut state = context.state.lock().unwrap();
        if state.revision == revision {
            state.frame = Some(frame);
        }
    }

    let (window, pid, monitor) = match source {
        CaptureSource::Monitor(monitor) => (0, 0, monitor),
        CaptureSource::Window(window) => (window.handle, window.process_id, 0),
    };
    let context = Context { stop, state };
    unsafe {
        lumatune_preview_run(
            window,
            pid,
            monitor,
            editor,
            (&raw const context).cast(),
            cancelled,
            settings,
            pixels,
        )
    }
}

#[cfg(not(all(target_env = "msvc", target_os = "windows")))]
fn run_preview(_: CaptureSource, _: usize, _: &AtomicBool, _: &Mutex<PreviewState>) -> i32 {
    windows::Win32::Foundation::E_NOTIMPL.0
}
