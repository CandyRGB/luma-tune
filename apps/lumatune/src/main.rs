use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use lumatune_core::{FilterParams, Preset, Ramp};
use lumatune_win::settings::{self, Settings, ThemeMode};
use lumatune_win::{Session, display, foreground, presets, state};
use slint::{ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel};

slint::include_modules!();

struct Runtime {
    params: FilterParams,
    settings: Settings,
    session: Option<Session>,
    presets: Vec<Preset>,
    active: Option<usize>,
    suppressing: bool,
    dirty: bool,
    comparing: bool,
}

fn main() -> Result<(), slint::PlatformError> {
    display::set_dpi_awareness();

    let displays = display::enumerate();
    let healed = state::heal(&displays);
    let target = display::primary(&displays).cloned();

    let ui = AppWindow::new()?;
    let prefs = settings::load();
    ui.set_theme_mode(prefs.theme.index());

    let mut notes: Vec<String> = Vec::new();
    if healed {
        notes.push("上次未正常退出，已把显示器还原为原始状态".into());
    }
    match &target {
        Some(d) => {
            ui.set_display_text(SharedString::from(format!(
                "{}  {}x{}  {}  {}",
                d.device,
                d.width,
                d.height,
                if d.hdr() { "HDR" } else { "SDR" },
                d.adapter
            )));
            if d.hdr() {
                notes.push("该显示器 HDR 已开启，查表被系统忽略，滤镜不会生效".into());
            } else if d.looks_virtual() {
                notes.push("检测到虚拟显示适配器，效果与真实显卡可能不一致".into());
            }
        }
        None => {
            ui.set_display_text(SharedString::from("未找到可用显示器"));
            notes.push("未找到可用显示器".into());
        }
    }
    ui.set_status(SharedString::from(notes.join("；")));

    let loaded = presets::list();
    fill_model(&ui, &loaded);
    fill_bindings(&ui, &loaded);

    let runtime = Rc::new(RefCell::new(Runtime {
        params: FilterParams::default(),
        settings: prefs,
        session: target.as_ref().and_then(Session::open),
        presets: loaded,
        active: None,
        suppressing: false,
        dirty: false,
        comparing: false,
    }));

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_changed(move || {
            let Some(ui) = weak.upgrade() else { return };
            if rt.borrow().suppressing {
                return;
            }
            let mut params = { rt.borrow().params.clone() };
            params.gamma = ui.get_gamma();
            params.contrast = ui.get_contrast();
            params.brightness = ui.get_brightness();
            params.shadows = ui.get_shadows();
            params.highlights = ui.get_highlights();
            params.levels.in_black = ui.get_black_point();
            params.levels.in_white = ui.get_white_point();
            params.levels.out_black = ui.get_black_level();
            params.levels.out_white = ui.get_white_level();
            params.temperature = ui.get_temperature() as u32;
            params.tint = ui.get_tint();
            params.gains = [ui.get_gain_r(), ui.get_gain_g(), ui.get_gain_b()];
            let mut rt = rt.borrow_mut();
            rt.params = params;
            rt.dirty = true;
        });
    }

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_toggle_compare(move || {
            let Some(ui) = weak.upgrade() else { return };
            let value = {
                let mut rt = rt.borrow_mut();
                rt.comparing = !rt.comparing;
                rt.dirty = true;
                rt.comparing
            };
            ui.set_comparing(value);
        });
    }

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_reset(move || {
            let Some(ui) = weak.upgrade() else { return };
            let params = FilterParams::default();
            {
                let mut rt = rt.borrow_mut();
                rt.params = params.clone();
                rt.comparing = false;
                rt.dirty = true;
                rt.suppressing = true;
            }
            push_params(&ui, &params);
            rt.borrow_mut().suppressing = false;
        });
    }

    {
        let rt = runtime.clone();
        ui.on_clear_selection(move || {
            let mut rt = rt.borrow_mut();
            rt.active = None;
        });
    }

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_pick_theme(move |mode| {
            let Some(ui) = weak.upgrade() else { return };
            let theme = ThemeMode::from_index(mode);
            let snapshot = {
                let mut rt = rt.borrow_mut();
                rt.settings.theme = theme;
                rt.settings.clone()
            };
            ui.set_theme_mode(theme.index());
            if let Err(err) = settings::save(&snapshot) {
                ui.set_status(SharedString::from(format!("主题已切换，但保存失败：{err}")));
            }
        });
    }

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_choose(move |index| {
            let Some(ui) = weak.upgrade() else { return };
            let chosen = {
                let rt = rt.borrow();
                rt.presets.get(index.max(0) as usize).cloned()
            };
            let Some(preset) = chosen else { return };
            {
                let mut rt = rt.borrow_mut();
                rt.params = preset.params.clone();
                rt.comparing = false;
                rt.dirty = true;
                rt.suppressing = true;
            }
            ui.set_preset_name(SharedString::from(preset.name.clone()));
            ui.set_active_name(SharedString::from(preset.name.clone()));
            push_params(&ui, &preset.params);
            rt.borrow_mut().suppressing = false;
            ui.set_status(SharedString::from(format!("已载入「{}」", preset.name)));
        });
    }

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_save_preset(move |name| {
            let Some(ui) = weak.upgrade() else { return };
            let name = name.trim().to_string();
            if name.is_empty() {
                ui.set_status(SharedString::from("预设名称不能为空"));
                return;
            }
            let params = {
                let rt = rt.borrow();
                rt.params.clone()
            };
            match presets::save(&Preset::new(name.clone(), params)) {
                Ok(path) => {
                    let list = {
                        let mut rt = rt.borrow_mut();
                        rt.presets = presets::list();
                        rt.presets.clone()
                    };
                    fill_model(&ui, &list);
                    fill_bindings(&ui, &list);
                    if let Some(pos) = list.iter().position(|p| p.name == name) {
                        ui.set_current_preset(pos as i32);
                    }
                    ui.set_active_name(SharedString::from(name.clone()));
                    ui.set_status(SharedString::from(format!(
                        "已保存到 {}",
                        path.display()
                    )));
                }
                Err(err) => {
                    ui.set_status(SharedString::from(format!("保存失败：{err}")));
                }
            }
        });
    }

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_delete_preset(move || {
            let Some(ui) = weak.upgrade() else { return };
            let index = ui.get_current_preset().max(0) as usize;
            let target = {
                let rt = rt.borrow();
                rt.presets.get(index).cloned()
            };
            let Some(preset) = target else {
                ui.set_status(SharedString::from("没有可删除的预设"));
                return;
            };
            match presets::remove(&preset.name) {
                Ok(()) => {
                    let list = {
                        let mut rt = rt.borrow_mut();
                        rt.presets = presets::list();
                        rt.active = None;
                        rt.presets.clone()
                    };
                    fill_model(&ui, &list);
                    fill_bindings(&ui, &list);
                    ui.set_current_preset(0);
                    ui.set_active_name(SharedString::from(""));
                    ui.set_preset_name(SharedString::from(""));
                    ui.set_status(SharedString::from(format!("已删除「{}」", preset.name)));
                }
                Err(err) => {
                    ui.set_status(SharedString::from(format!("删除失败：{err}")));
                }
            }
        });
    }

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_bind_foreground(move || {
            let Some(ui) = weak.upgrade() else { return };
            let Some(fg) = foreground::current() else {
                ui.set_status(SharedString::from("取不到前台窗口"));
                return;
            };
            if fg.exe.eq_ignore_ascii_case("lumatune.exe") {
                ui.set_status(SharedString::from("不能绑定本程序自己，先切到目标应用再点"));
                return;
            }
            let index = ui.get_current_preset().max(0) as usize;
            let updated = {
                let mut rt = rt.borrow_mut();
                let Some(mut preset) = rt.presets.get(index).cloned() else {
                    return;
                };
                preset.bind_to(fg.exe.clone());
                if presets::save(&preset).is_err() {
                    return;
                }
                rt.presets = presets::list();
                rt.presets.clone()
            };
            fill_bindings(&ui, &updated);
            ui.set_status(SharedString::from(format!(
                "已把「{}」绑定到 {}",
                ui.get_preset_name(),
                fg.exe
            )));
        });
    }

    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        ui.on_unbind_foreground(move || {
            let Some(ui) = weak.upgrade() else { return };
            let Some(fg) = foreground::current() else {
                return;
            };
            let updated = {
                let mut rt = rt.borrow_mut();
                let mut touched = false;
                for preset in rt.presets.iter_mut() {
                    if preset.matches_exe(&fg.exe) {
                        preset.unbind(&fg.exe);
                        let _ = presets::save(preset);
                        touched = true;
                    }
                }
                if !touched {
                    return;
                }
                rt.active = None;
                rt.presets = presets::list();
                rt.presets.clone()
            };
            fill_bindings(&ui, &updated);
            ui.set_status(SharedString::from(format!("已解除 {}", fg.exe)));
        });
    }

    let writer = Timer::default();
    {
        let rt = runtime.clone();
        writer.start(TimerMode::Repeated, Duration::from_millis(30), move || {
            let mut rt = rt.borrow_mut();
            if !rt.dirty {
                return;
            }
            rt.dirty = false;
            let table: Option<Ramp> = if rt.comparing {
                None
            } else if let Some(index) = rt.active {
                rt.presets.get(index).map(|p| p.params.to_ramp())
            } else if rt.params.is_neutral() {
                None
            } else {
                Some(rt.params.to_ramp())
            };
            if let Some(session) = rt.session.as_mut() {
                match table {
                    Some(ramp) => {
                        session.apply(&ramp);
                    }
                    None => session.restore(),
                }
            }
        });
    }

    let watcher = Timer::default();
    {
        let rt = runtime.clone();
        let weak = ui.as_weak();
        watcher.start(TimerMode::Repeated, Duration::from_millis(400), move || {
            let Some(ui) = weak.upgrade() else { return };
            let Some(fg) = foreground::current() else { return };
            let switched = {
                let mut rt = rt.borrow_mut();
                let matched = rt.presets.iter().position(|p| p.matches_exe(&fg.exe));
                if matched == rt.active {
                    return;
                }
                rt.active = matched;
                rt.dirty = true;
                match matched {
                    Some(index) => Some((
                        rt.presets[index].params.clone(),
                        rt.presets[index].name.clone(),
                    )),
                    None => None,
                }
            };
            let (params, label) = match switched {
                Some((params, name)) => (params, format!("「{name}」接管中")),
                None => {
                    let params = rt.borrow().params.clone();
                    (params, String::new())
                }
            };
            rt.borrow_mut().suppressing = true;
            push_params(&ui, &params);
            rt.borrow_mut().suppressing = false;
            ui.set_status(SharedString::from(label));
        });
    }

    install_snapshot_hook(&ui);

    let bench = Timer::default();
    if std::env::var("LUMATUNE_BENCH_NAV").is_ok() {
        let weak = ui.as_weak();
        let mut tick = 0i32;
        bench.start(TimerMode::Repeated, Duration::from_millis(250), move || {
            tick += 1;
            if let Some(ui) = weak.upgrade() {
                ui.set_nav_compact(tick % 2 == 0);
            }
            if tick >= 12 {
                let _ = slint::quit_event_loop();
            }
        });
    }

    let result = ui.run();
    bench.stop();

    if let Some(session) = runtime.borrow_mut().session.as_mut() {
        session.restore();
    }
    drop(writer);
    drop(watcher);
    result
}

fn install_snapshot_hook(ui: &AppWindow) {
    let Ok(path) = std::env::var("LUMATUNE_SHOT") else {
        return;
    };
    let anim = std::env::var("LUMATUNE_SHOT_ANIM").is_ok();

    let prep = ui.as_weak();
    slint::Timer::single_shot(Duration::from_millis(350), move || {
        if let Some(ui) = prep.upgrade() {
            if std::env::var("LUMATUNE_SHOT_EXPANDED").is_ok() {
                ui.set_light_expanded(true);
                ui.set_color_expanded(true);
            }
            if let Ok(modes) = std::env::var("LUMATUNE_SHOT_THEME") {
                for mode in modes.split(',') {
                    if let Ok(index) = mode.trim().parse::<i32>() {
                        ui.set_theme_mode(index);
                    }
                }
            }
            if let Ok(name) = std::env::var("LUMATUNE_SHOT_PRESET") {
                ui.set_preset_name(SharedString::from(name.clone()));
                ui.set_active_name(SharedString::from(name));
            }
            if std::env::var("LUMATUNE_SHOT_COMPACT").is_ok() {
                ui.set_nav_compact(true);
            }
            if let Ok(page) = std::env::var("LUMATUNE_SHOT_PAGE") {
                if let Ok(index) = page.parse::<i32>() {
                    ui.set_page(index);
                }
            }
        }
    });

    if anim {
        let toggle = ui.as_weak();
        slint::Timer::single_shot(Duration::from_millis(500), move || {
            if let Some(ui) = toggle.upgrade() {
                ui.set_nav_compact(!ui.get_nav_compact());
            }
        });
    }

    let delay = if anim { 590 } else { 700 };
    let weak = ui.as_weak();
    slint::Timer::single_shot(Duration::from_millis(delay), move || {
        if let Some(ui) = weak.upgrade() {
            match ui.window().take_snapshot() {
                Ok(buffer) => match write_bmp(std::path::Path::new(&path), &buffer) {
                    Ok(()) => eprintln!("snapshot written to {path}"),
                    Err(err) => eprintln!("snapshot write failed: {err}"),
                },
                Err(err) => eprintln!("snapshot failed: {err}"),
            }
        }
        let _ = slint::quit_event_loop();
    });
}
fn write_bmp(
    path: &std::path::Path,
    buffer: &slint::SharedPixelBuffer<slint::Rgba8Pixel>,
) -> std::io::Result<()> {
    let width = buffer.width() as usize;
    let height = buffer.height() as usize;
    let stride = width * 4;
    let image_size = stride * height;
    let file_size = 54 + image_size;

    let mut out: Vec<u8> = Vec::with_capacity(file_size);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(file_size as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(width as i32).to_le_bytes());
    out.extend_from_slice(&(height as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(image_size as u32).to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());

    let pixels = buffer.as_slice();
    for row in (0..height).rev() {
        for pixel in &pixels[row * width..(row + 1) * width] {
            out.push(pixel.b);
            out.push(pixel.g);
            out.push(pixel.r);
            out.push(255);
        }
    }
    std::fs::write(path, out)
}

fn fill_model(ui: &AppWindow, list: &[Preset]) {
    let names: Vec<SharedString> = list
        .iter()
        .map(|p| SharedString::from(p.name.clone()))
        .collect();
    let model = Rc::new(VecModel::from(names));
    ui.set_presets(ModelRc::from(model));
}

fn fill_bindings(ui: &AppWindow, list: &[Preset]) {
    let rows: Vec<SharedString> = list
        .iter()
        .map(|p| {
            let bound = if p.bindings.is_empty() {
                "未绑定".to_string()
            } else {
                p.bindings.join("、")
            };
            SharedString::from(format!("{}　{}", p.name, bound))
        })
        .collect();
    let model = Rc::new(VecModel::from(rows));
    ui.set_binding_rows(ModelRc::from(model));
}

fn push_params(ui: &AppWindow, params: &FilterParams) {
    ui.set_gamma(params.gamma);
    ui.set_contrast(params.contrast);
    ui.set_brightness(params.brightness);
    ui.set_shadows(params.shadows);
    ui.set_highlights(params.highlights);
    ui.set_black_point(params.levels.in_black);
    ui.set_white_point(params.levels.in_white);
    ui.set_black_level(params.levels.out_black);
    ui.set_white_level(params.levels.out_white);
    ui.set_temperature(params.temperature as f32);
    ui.set_tint(params.tint);
    ui.set_gain_r(params.gains[0]);
    ui.set_gain_g(params.gains[1]);
    ui.set_gain_b(params.gains[2]);
    ui.set_comparing(false);
}
