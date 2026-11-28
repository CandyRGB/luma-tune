use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::num::NonZeroIsize;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use lumatune_core::{FilterParams, PostProcessParams, Preset, Ramp};
use lumatune_win::postprocess;
use lumatune_win::presets::StoredPreset;
use lumatune_win::settings::{self, DesktopPreset, Settings, ThemeMode};
use lumatune_win::{app_icons, display, file_picker, foreground, presets, state};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use slint::winit_030::{EventResult, WinitWindowAccessor, winit::event::WindowEvent};
use slint::{
    ComponentHandle, Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, Timer,
    TimerMode, VecModel,
};
use slint_winui::platform::title_bar;

mod contrast_theme;
mod desktop;

slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/slint_winui_behaviors.rs"));

struct Draft {
    index: Option<usize>,
    params: FilterParams,
}

struct Runtime {
    presets: Vec<StoredPreset>,
    application_icons: HashMap<String, Image>,
    settings: Settings,
    screens: Vec<desktop::Screen>,
    active: Option<usize>,
    foreground: String,
    foreground_monitor: usize,
    draft: Option<Draft>,
    dirty: bool,
}

impl Runtime {
    fn refresh_displays(&mut self, displays: Vec<display::Display>) -> bool {
        if !desktop::reconcile(&mut self.screens, displays, &self.settings) {
            return false;
        }
        self.dirty = true;
        true
    }

    fn current_ramp(&self, screen: &desktop::Screen) -> Option<Ramp> {
        let foreground = self
            .active
            .map(|index| (self.foreground_monitor, &self.presets[index].preset.params));
        screen.desired_ramp(foreground)
    }

    fn set_desktop(&mut self, display: &str, preset: Option<usize>) -> io::Result<()> {
        let screen = self
            .screens
            .iter_mut()
            .find(|screen| screen.display.identity.as_deref() == Some(display) && screen.ready())
            .ok_or_else(|| invalid("这台显示器当前无法调校"))?;
        let selection = preset.map(|index| DesktopPreset {
            preset_id: self.presets[index].id(),
            params: self.presets[index].preset.params.clone(),
        });
        let mut updated = self.settings.clone();
        match &selection {
            Some(preset) => {
                updated.desktop.insert(display.to_owned(), preset.clone());
            }
            None => {
                updated.desktop.remove(display);
            }
        }
        settings::save(&updated)?;
        self.settings = updated;
        screen.desktop = selection;
        self.dirty = true;
        Ok(())
    }

    fn refresh_active(&mut self) {
        let next = self
            .presets
            .iter()
            .position(|entry| entry.preset.matches_exe(&self.foreground));
        self.dirty |= self.active != next;
        self.active = next;
    }

    fn forget_cover(&self, name: Option<&str>) {
        let Some(name) = name else { return };
        if self.presets.iter().any(|entry| {
            entry
                .preset
                .cover
                .as_deref()
                .is_some_and(|cover| cover.eq_ignore_ascii_case(name))
        }) {
            return;
        }
        if let Some(path) = presets::cover_path(name) {
            let _ = std::fs::remove_file(path);
        }
    }

    fn save_draft(
        &mut self,
        name: &str,
        cover: &CoverChoice,
        post_process: PostProcessParams,
    ) -> io::Result<()> {
        let draft = self
            .draft
            .as_ref()
            .ok_or_else(|| invalid("没有正在编辑的预设"))?;
        let name = name.trim();
        if name.is_empty() {
            return Err(invalid("请输入预设名称"));
        }
        let previous = draft.index.and_then(|index| self.presets.get(index));
        let renamed = previous.is_none_or(|entry| entry.preset.name != name);
        if renamed
            && self
                .presets
                .iter()
                .any(|entry| entry.preset.name.to_lowercase() == name.to_lowercase())
        {
            return Err(invalid("此名称已被其他预设使用"));
        }
        let mut updated = previous
            .map(|entry| entry.preset.clone())
            .unwrap_or_else(|| Preset::new(name, FilterParams::default()));
        updated.name = name.into();
        updated.params = draft.params.clone();
        updated.post_process = post_process.normalized();
        let old_cover = updated.cover.clone();
        let imported = if cover.changed && !cover.source.is_empty() {
            let name = presets::import_cover(Path::new(cover.source.as_str()))?;
            let readable =
                presets::cover_path(&name).is_some_and(|path| Image::load_from_path(&path).is_ok());
            if !readable {
                self.forget_cover(Some(&name));
                return Err(invalid("无法读取所选封面"));
            }
            Some(name)
        } else {
            None
        };
        if cover.changed {
            updated.cover = imported.clone();
        }
        let saved = if let Some(index) = draft.index {
            self.presets[index].save(updated)
        } else {
            presets::create(updated).map(|entry| self.presets.push(entry))
        };
        match saved {
            Ok(()) => {
                self.dirty = true;
                if cover.changed {
                    self.forget_cover(old_cover.as_deref());
                }
                Ok(())
            }
            Err(error) => {
                self.forget_cover(imported.as_deref());
                Err(error)
            }
        }
    }

    fn add_binding(&mut self, application: &str, index: i32) -> io::Result<()> {
        presets::recover_binding_move(&mut self.presets)?;
        let application = application.trim();
        let path = Path::new(application);
        let source = path.is_absolute().then_some(path);
        let name = match source {
            Some(path) => path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
            None => application,
        };
        if !name.to_ascii_lowercase().ends_with(".exe")
            || name.len() <= 4
            || name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
            || name.chars().any(char::is_control)
            || name.eq_ignore_ascii_case("lumatune.exe")
        {
            return Err(invalid("请输入目标程序的 .exe 文件名"));
        }
        if source.is_some_and(|path| !path.is_file()) {
            return Err(invalid("找不到指定的应用程序文件"));
        }
        let index = usize::try_from(index).map_err(|_| invalid("请选择关联预设"))?;
        if let Some(entry) =
            self.presets.iter().enumerate().find_map(|(i, entry)| {
                (i != index && entry.preset.matches_exe(name)).then_some(entry)
            })
        {
            return Err(invalid(format!(
                "该应用已绑定到「{}」，请先解除原绑定",
                entry.preset.name
            )));
        }
        let entry = self
            .presets
            .get_mut(index)
            .ok_or_else(|| invalid("请选择关联预设"))?;
        let mut updated = entry.preset.clone();
        updated.bind_to(name);
        entry.save(updated)?;
        if let Some(path) = source {
            self.clear_application_icon(name);
            self.capture_application_icon(name, path);
        }
        self.refresh_active();
        Ok(())
    }

    fn remove_binding(&mut self, name: &str, index: i32) -> io::Result<()> {
        presets::recover_binding_move(&mut self.presets)?;
        let entry = usize::try_from(index)
            .ok()
            .and_then(|i| self.presets.get_mut(i))
            .ok_or_else(|| invalid("找不到该绑定"))?;
        let mut updated = entry.preset.clone();
        updated.unbind(name);
        entry.save(updated)?;
        self.forget_application_icon(name);
        self.refresh_active();
        Ok(())
    }

    fn rebind_application(&mut self, name: &str, index: i32) -> io::Result<()> {
        presets::recover_binding_move(&mut self.presets)?;
        let target = usize::try_from(index).map_err(|_| invalid("请选择关联预设"))?;
        let current = self
            .presets
            .iter()
            .position(|entry| entry.preset.matches_exe(name))
            .ok_or_else(|| invalid("找不到该绑定"))?;
        if target == current {
            return Ok(());
        }
        let [from, to] = self
            .presets
            .get_disjoint_mut([current, target])
            .map_err(|_| invalid("找不到关联预设"))?;
        presets::rebind(from, to, name)?;
        self.refresh_active();
        Ok(())
    }

    fn capture_application_icon(&mut self, name: &str, source: &Path) {
        if let Some(icon) = app_icons::capture(name, source) {
            let pixels = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                &icon.rgba,
                icon.width,
                icon.height,
            );
            self.application_icons
                .insert(name.to_ascii_lowercase(), Image::from_rgba8(pixels));
        }
    }

    fn clear_application_icon(&mut self, name: &str) {
        self.application_icons.remove(&name.to_ascii_lowercase());
        if let Some(path) = app_icons::cache_path(name) {
            let _ = std::fs::remove_file(path);
        }
    }

    fn forget_application_icon(&mut self, name: &str) {
        if !self
            .presets
            .iter()
            .any(|entry| entry.preset.matches_exe(name))
        {
            self.clear_application_icon(name);
        }
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn main() -> Result<(), slint::PlatformError> {
    display::set_dpi_awareness();
    let displays = display::enumerate();
    let ui = AppWindow::new()?;
    breadcrumb_bar::install(ui.global::<BreadcrumbBarBehavior>());
    combo_box::install(ui.global::<ComboBoxBehavior>());
    navigation_view::install(ui.global::<NavigationViewBehavior>(), &ui);
    number_box::install(ui.global::<NumberBoxMath>());
    radio_buttons::install(ui.global::<RadioButtonsBehavior>());
    let _display_lock = match state::claim() {
        Ok(lock) => lock,
        Err(error) => {
            ui.invoke_notify(
                InfoSeverity::Error,
                "无法启动显示调校".into(),
                format!("请确认只有一个 LumaTune 实例在运行。{error}").into(),
            );
            return run_window(&ui);
        }
    };

    let _contrast_watcher = contrast_theme::watch(&ui);

    let mut saved_presets = presets::list();
    if let Err(error) = presets::recover_binding_move(&mut saved_presets) {
        ui.invoke_notify(
            InfoSeverity::Error,
            "应用绑定恢复失败".into(),
            error.to_string().into(),
        );
    }
    let mut prefs = settings::load();
    prefs.desktop.retain(|_, selected| {
        saved_presets
            .iter()
            .any(|entry| entry.id() == selected.preset_id)
    });
    ui.set_theme_mode(prefs.theme.index());
    let application_icons = saved_presets
        .iter()
        .flat_map(|entry| &entry.preset.bindings)
        .filter_map(|exe| {
            let path = app_icons::cache_path(exe)?;
            if !path.is_file() {
                return None;
            }
            let icon = Image::load_from_path(&path).ok()?;
            Some((exe.to_ascii_lowercase(), icon))
        })
        .collect();
    let mut rt = Runtime {
        presets: saved_presets,
        application_icons,
        settings: prefs,
        screens: Vec::new(),
        active: None,
        foreground: String::new(),
        foreground_monitor: 0,
        draft: None,
        dirty: true,
    };
    rt.refresh_displays(displays);
    let runtime = Rc::new(RefCell::new(rt));
    fill_library(&ui, &runtime.borrow());
    install_editor(&ui, &runtime);
    let preview = install_preview(&ui, &runtime);
    install_bindings(&ui, &runtime);
    install_desktop(&ui, &runtime);

    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_pick_theme(move |index| {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        let theme = ThemeMode::from_index(index);
        if rt.settings.theme == theme {
            return;
        }
        let mut prefs = rt.settings.clone();
        prefs.theme = theme;
        if let Err(error) = settings::save(&prefs) {
            ui.set_theme_mode(rt.settings.theme.index());
            ui.invoke_notify(
                InfoSeverity::Error,
                "主题保存失败".into(),
                error.to_string().into(),
            );
        } else {
            rt.settings = prefs;
        }
    });

    let writer = Timer::default();
    let rt = runtime.clone();
    let weak = ui.as_weak();
    writer.start(TimerMode::Repeated, Duration::from_millis(30), move || {
        let mut rt = rt.borrow_mut();
        if !std::mem::take(&mut rt.dirty) {
            return;
        }
        let ramps: Vec<_> = rt
            .screens
            .iter()
            .map(|screen| rt.current_ramp(screen))
            .collect();
        for (screen, ramp) in rt.screens.iter_mut().zip(ramps) {
            if !screen.render(ramp)
                && screen.first_failure()
                && let Some(ui) = weak.upgrade()
            {
                ui.invoke_notify(
                    InfoSeverity::Warning,
                    "显示器调校失败".into(),
                    screen.display.name.as_str().into(),
                );
            }
        }
    });

    let displays = Timer::default();
    let weak = ui.as_weak();
    let rt = runtime.clone();
    displays.start(TimerMode::Repeated, Duration::from_secs(2), move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        rt.dirty |= rt.screens.iter().any(desktop::Screen::needs_retry);
        if rt.refresh_displays(display::enumerate()) {
            fill_desktop(&ui, &rt);
        }
    });

    let watcher = Timer::default();
    let weak = ui.as_weak();
    let mut processor = postprocess::Processor::default();
    let rt = runtime.clone();
    watcher.start(TimerMode::Repeated, Duration::from_millis(400), move || {
        let Some(ui) = weak.upgrade() else { return };
        let fg = foreground::current();
        let exe = fg.as_ref().map_or("", |fg| fg.exe.as_str());
        let mut rt = rt.borrow_mut();
        let monitor = fg.as_ref().map_or(0, |fg| fg.monitor);
        let moved = rt.foreground_monitor != monitor;
        rt.foreground_monitor = monitor;
        rt.dirty |= moved && rt.active.is_some();
        let previous = rt.active;
        if !rt.foreground.eq_ignore_ascii_case(exe) {
            if let Some(fg) = &fg
                && rt.presets.iter().any(|entry| entry.preset.matches_exe(exe))
                && !rt.application_icons.contains_key(&exe.to_ascii_lowercase())
            {
                rt.capture_application_icon(exe, Path::new(&fg.path));
                fill_bindings(&ui, &rt);
            }
            rt.foreground = exe.to_owned();
            rt.refresh_active();
            if rt.active != previous {
                if let Some(index) = rt.active {
                    ui.invoke_notify_foreground(
                        "已匹配应用预设".into(),
                        format!(
                            "{} 正在使用「{}」。",
                            rt.foreground, rt.presets[index].preset.name
                        )
                        .into(),
                    );
                } else {
                    ui.invoke_notify_foreground("已恢复桌面调校".into(), "".into());
                }
            }
        }
        if previous != rt.active || moved && rt.active.is_some() {
            fill_desktop(&ui, &rt);
        }
        let post_process = rt.active.zip(fg).and_then(|(index, fg)| {
            postprocess::Target::new(fg.window, rt.presets[index].preset.post_process)
        });
        if let Err(message) = processor.update(post_process) {
            ui.invoke_notify(
                InfoSeverity::Warning,
                "画面后处理不可用".into(),
                message.into(),
            );
        }
    });

    let result = run_window(&ui);
    ui.invoke_select_capture_source(-1);
    drop(preview);
    drop(watcher);
    drop(displays);
    drop(writer);
    let mut rt = runtime.borrow_mut();
    rt.refresh_displays(display::enumerate());
    rt.screens.clear();
    result
}

fn run_window(ui: &AppWindow) -> Result<(), slint::PlatformError> {
    let weak = ui.as_weak();
    let _ = slint::spawn_local(async move {
        if let Some(ui) = weak.upgrade()
            && ui.window().winit_window().await.is_ok()
            && let Some(hwnd) = window_handle(&ui)
        {
            let caption_height = ui.get_preferred_caption_height();
            if let Err(error) = window_title_bar::attach(
                ui.global::<WindowTitleBarBehavior>(),
                hwnd,
                caption_height as u16,
            ) {
                eprintln!("Could not attach native title bar: {error}");
                return;
            }
            ui.set_caption_height(caption_height);
            ui.set_maximized_caption_inset(title_bar::maximized_inset(hwnd));
            update_caption_colors(&ui, hwnd);
            let weak = ui.as_weak();
            ui.on_caption_colors_changed(move || {
                if let Some(ui) = weak.upgrade() {
                    update_caption_colors(&ui, hwnd);
                }
            });
            let weak = ui.as_weak();
            ui.window().on_winit_window_event(move |_, event| {
                if matches!(event, WindowEvent::ScaleFactorChanged { .. })
                    && let Some(ui) = weak.upgrade()
                {
                    ui.set_maximized_caption_inset(title_bar::maximized_inset(hwnd));
                }
                EventResult::Propagate
            });
        }
    });
    ui.run()
}

fn update_caption_colors(ui: &AppWindow, hwnd: NonZeroIsize) {
    let color_ref = |color: slint::Color| {
        u32::from_le_bytes([color.red(), color.green(), color.blue(), 0])
    };
    title_bar::set_colors(
        hwnd,
        color_ref(ui.get_caption_background()),
        color_ref(ui.get_caption_foreground()),
    );
}

fn window_handle(ui: &AppWindow) -> Option<NonZeroIsize> {
    let window = ui.window().window_handle();
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd),
        _ => None,
    }
}

fn install_editor(ui: &AppWindow, runtime: &Rc<RefCell<Runtime>>) {
    ui.on_filter_presets(|items, query, only_favorites| {
        let query = query.trim().to_lowercase();
        let cards: Vec<_> = items
            .iter()
            .filter(|card| {
                (!only_favorites || card.favorite) && card.name.to_lowercase().contains(&query)
            })
            .collect();
        model(cards)
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_open_preset(move |index| {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        let index = usize::try_from(index)
            .ok()
            .filter(|&i| i < rt.presets.len());
        let entry = index.map(|i| &rt.presets[i]);
        let params = entry.map(|e| e.preset.params.clone()).unwrap_or_default();
        let post_process = entry
            .map(|e| e.preset.post_process.normalized())
            .unwrap_or_default();
        let name = entry
            .map(|e| SharedString::from(&e.preset.name))
            .unwrap_or_default();
        let cover = entry.map(|e| load_cover(&e.preset)).unwrap_or_default();
        ui.set_existing_draft(entry.is_some());
        ui.set_built_in_draft(entry.is_some_and(|e| e.defaults.is_some()));
        ui.set_preset_name(name.clone());
        ui.set_original_name(name);
        ui.set_edit_cover(CoverChoice {
            artwork: cover,
            ..CoverChoice::default()
        });
        push_params(&ui, &params);
        push_post_process(&ui, post_process);
        rt.draft = Some(Draft { index, params });
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_parameters_edited(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        if let Some(draft) = rt.draft.as_mut() {
            read_params(&ui, &mut draft.params);
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_restore_defaults(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        let defaults = rt
            .draft
            .as_ref()
            .and_then(|draft| draft.index)
            .and_then(|i| rt.presets[i].defaults.clone());
        if let (Some(draft), Some(params)) = (rt.draft.as_mut(), defaults) {
            push_params(&ui, &params);
            push_post_process(&ui, PostProcessParams::default());
            draft.params = params;
        }
    });
    let rt = runtime.clone();
    ui.on_close_editor(move || rt.borrow_mut().draft = None);
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_save_preset(move |name, cover| {
        let Some(ui) = weak.upgrade() else {
            return false;
        };
        let mut rt = rt.borrow_mut();
        match rt.save_draft(name.as_str(), &cover, read_post_process(&ui)) {
            Ok(()) => {
                fill_library(&ui, &rt);
                ui.invoke_notify(
                    InfoSeverity::Success,
                    "已保存预设".into(),
                    name.trim().into(),
                );
                true
            }
            Err(error) => {
                ui.invoke_notify(
                    InfoSeverity::Error,
                    "保存失败".into(),
                    error.to_string().into(),
                );
                false
            }
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_toggle_favorite(move |index| {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        let Some(entry) = usize::try_from(index)
            .ok()
            .and_then(|i| rt.presets.get_mut(i))
        else {
            return;
        };
        let mut updated = entry.preset.clone();
        updated.favorite = !updated.favorite;
        match entry.save(updated) {
            Ok(()) => fill_cards(&ui, &rt),
            Err(error) => ui.invoke_notify(
                InfoSeverity::Error,
                "收藏保存失败".into(),
                error.to_string().into(),
            ),
        }
    });
    let weak = ui.as_weak();
    ui.on_browse_cover(move || {
        let Some(ui) = weak.upgrade() else {
            return CoverChoice::default();
        };
        match file_picker::cover() {
            Ok(Some(path)) => match Image::load_from_path(&path) {
                Ok(artwork) => CoverChoice {
                    artwork,
                    source: path.to_string_lossy().as_ref().into(),
                    changed: true,
                },
                Err(error) => {
                    ui.invoke_notify(
                        InfoSeverity::Error,
                        "无法打开图片".into(),
                        error.to_string().into(),
                    );
                    CoverChoice::default()
                }
            },
            Ok(None) => CoverChoice::default(),
            Err(error) => {
                ui.invoke_notify(
                    InfoSeverity::Error,
                    "无法选择图片".into(),
                    error.to_string().into(),
                );
                CoverChoice::default()
            }
        }
    });
}

#[derive(Default)]
struct EditorPreview {
    sources: Vec<postprocess::CaptureSource>,
    selected: Option<postprocess::CaptureSource>,
    worker: Option<postprocess::Preview>,
    display_params: Option<FilterParams>,
    ramp: Option<Ramp>,
}

fn clear_preview(ui: &AppWindow, status: &str) {
    ui.set_preview_aspect(0.0);
    ui.set_preview_before(Image::default());
    ui.set_preview_after(Image::default());
    ui.set_preview_status(status.into());
}

fn install_preview(ui: &AppWindow, runtime: &Rc<RefCell<Runtime>>) -> Timer {
    use postprocess::CaptureSource;
    let preview = Rc::new(RefCell::new(EditorPreview::default()));
    let weak = ui.as_weak();
    let capture = preview.clone();
    let rt = runtime.clone();
    ui.on_refresh_capture_sources(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut choices: Vec<_> = rt
            .borrow()
            .screens
            .iter()
            .map(|screen| {
                let display = &screen.display;
                (CaptureSource::Monitor(display.handle), display.name.clone())
            })
            .collect();
        choices.extend(foreground::capture_windows().into_iter().map(|choice| {
            let group = if choice.fullscreen {
                "全屏应用"
            } else {
                "窗口"
            };
            (
                CaptureSource::Window(choice.window),
                format!("{group} · {}", choice.title),
            )
        }));
        let (index, removed) = {
            let mut capture = capture.borrow_mut();
            capture.sources = choices.iter().map(|(source, _)| *source).collect();
            let index = capture
                .selected
                .and_then(|selection| choices.iter().position(|(source, _)| *source == selection));
            (index, capture.selected.is_some() && index.is_none())
        };
        if removed {
            ui.invoke_select_capture_source(-1);
        }
        ui.set_capture_index(-1);
        ui.set_capture_sources(model(
            choices
                .into_iter()
                .map(|(_, text)| ComboBoxItem {
                    text: text.into(),
                    ..Default::default()
                })
                .collect(),
        ));
        ui.set_capture_index(index.map_or(-1, |index| index as i32));
    });

    let weak = ui.as_weak();
    let capture = preview.clone();
    ui.on_select_capture_source(move |index| {
        let Some(ui) = weak.upgrade() else { return };
        let mut capture = capture.borrow_mut();
        let selected = usize::try_from(index)
            .ok()
            .and_then(|index| capture.sources.get(index).copied());
        if capture.selected == selected {
            return;
        }
        capture.worker = None;
        capture.selected = selected;
        capture.display_params = None;
        capture.ramp = None;
        ui.set_capturing(false);
        if let Some(source) = selected {
            let editor = window_handle(&ui).map_or(0, |hwnd| hwnd.get() as usize);
            match postprocess::Preview::start(source, editor) {
                Ok(worker) => {
                    capture.worker = Some(worker);
                    ui.set_capturing(true);
                    clear_preview(&ui, "正在获取画面…");
                }
                Err(error) => {
                    capture.selected = None;
                    clear_preview(&ui, &format!("无法启动预览：{error}"));
                    ui.set_capture_index(-1);
                }
            }
        } else {
            clear_preview(&ui, "选择显示器或窗口开始预览");
            ui.set_capture_index(-1);
        }
    });

    let timer = Timer::default();
    let weak = ui.as_weak();
    let rt = runtime.clone();
    timer.start(TimerMode::Repeated, Duration::from_millis(50), move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut capture = preview.borrow_mut();
        if capture.worker.is_none() {
            return;
        }
        let rt = rt.borrow();
        let Some(draft) = &rt.draft else { return };
        if capture.display_params.as_ref() != Some(&draft.params) {
            capture.ramp = (!draft.params.is_neutral()).then(|| draft.params.to_ramp());
            capture.display_params = Some(draft.params.clone());
        }
        let EditorPreview { worker, ramp, .. } = &mut *capture;
        let Some(worker) = worker else { return };
        worker.update(read_post_process(&ui).normalized(), *ramp);
        if let Some(status) = worker.finished() {
            capture.worker = None;
            capture.selected = None;
            ui.set_capturing(false);
            clear_preview(&ui, &status);
            ui.set_capture_index(-1);
        } else if let Some(frame) = worker.frame() {
            let image = |data: &[u8]| {
                Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    data,
                    frame.width,
                    frame.height,
                ))
            };
            ui.set_preview_before(image(&frame.before));
            ui.set_preview_after(image(&frame.after));
            ui.set_preview_aspect(frame.width as f32 / frame.height as f32);
        }
    });
    timer
}

fn install_bindings(ui: &AppWindow, runtime: &Rc<RefCell<Runtime>>) {
    let weak = ui.as_weak();
    ui.on_browse_application(move || {
        let Some(ui) = weak.upgrade() else {
            return SharedString::default();
        };
        match file_picker::executable() {
            Ok(Some(path)) => path.to_string_lossy().as_ref().into(),
            Ok(None) => SharedString::default(),
            Err(error) => {
                ui.invoke_notify(
                    InfoSeverity::Error,
                    "无法选择程序".into(),
                    error.to_string().into(),
                );
                SharedString::default()
            }
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_save_binding(move |exe, index| {
        let Some(ui) = weak.upgrade() else {
            return false;
        };
        let mut rt = rt.borrow_mut();
        match rt.add_binding(exe.as_str(), index) {
            Ok(()) => {
                fill_bindings(&ui, &rt);
                ui.invoke_notify(InfoSeverity::Success, "已保存应用绑定".into(), "".into());
                true
            }
            Err(error) => {
                ui.invoke_notify(
                    InfoSeverity::Error,
                    "绑定失败".into(),
                    error.to_string().into(),
                );
                false
            }
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_remove_binding(move |exe, index| {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        match rt.remove_binding(exe.as_str(), index) {
            Ok(()) => {
                fill_bindings(&ui, &rt);
                ui.invoke_notify(InfoSeverity::Success, "已解除应用绑定".into(), "".into());
            }
            Err(error) => {
                ui.invoke_notify(
                    InfoSeverity::Error,
                    "解除失败".into(),
                    error.to_string().into(),
                );
            }
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_rebind_application(move |exe, index| {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        match rt.rebind_application(exe.as_str(), index) {
            Ok(()) => {
                let rows = ui.get_bindings();
                if let Some((row_index, mut row)) = rows
                    .iter()
                    .enumerate()
                    .find(|(_, row)| row.executable == exe)
                {
                    row.index = index;
                    rows.set_row_data(row_index, row);
                }
                fill_desktop(&ui, &rt);
            }
            Err(error) => {
                fill_bindings(&ui, &rt);
                ui.invoke_notify(
                    InfoSeverity::Error,
                    "关联预设保存失败".into(),
                    error.to_string().into(),
                );
            }
        }
    });
}

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

fn load_cover(preset: &Preset) -> Image {
    preset
        .cover
        .as_deref()
        .and_then(presets::cover_path)
        .and_then(|path| Image::load_from_path(&path).ok())
        .unwrap_or_default()
}

fn fill_cards(ui: &AppWindow, rt: &Runtime) {
    let (built_in, custom): (Vec<_>, Vec<_>) = rt
        .presets
        .iter()
        .enumerate()
        .partition(|(_, entry)| entry.defaults.is_some());
    ui.set_presets(model(
        built_in
            .into_iter()
            .chain(custom)
            .map(|(index, entry)| PresetCardData {
                index: index as i32,
                name: entry.preset.name.as_str().into(),
                cover: load_cover(&entry.preset),
                favorite: entry.preset.favorite,
            })
            .collect(),
    ));
}

fn fill_library(ui: &AppWindow, rt: &Runtime) {
    fill_cards(ui, rt);
    fill_desktop(ui, rt);
    fill_bindings(ui, rt);
    ui.set_binding_presets(model(
        rt.presets
            .iter()
            .map(|entry| ComboBoxItem {
                text: entry.preset.name.as_str().into(),
                ..ComboBoxItem::default()
            })
            .collect(),
    ));
}

fn fill_desktop(ui: &AppWindow, rt: &Runtime) {
    let screen_rows = rt
        .screens
        .iter()
        .map(|screen| {
            let display = &screen.display;
            let selected = screen.desktop.as_ref().and_then(|preset| {
                rt.presets
                    .iter()
                    .position(|entry| entry.id() == preset.preset_id)
            });
            let status = if display.hdr() {
                "HDR · 调校不可用"
            } else if display.identity.is_none() {
                "无法识别显示器 · 调校不可用"
            } else if screen.ready() {
                "SDR"
            } else {
                "调校不可用"
            };
            let foreground = rt
                .active
                .filter(|_| rt.foreground_monitor == display.handle && screen.ready())
                .map(|index| rt.presets[index].preset.name.as_str())
                .unwrap_or_default();
            DisplayCard {
                id: display.key().into(),
                title: display.name.as_str().into(),
                detail: format!(
                    "{}{status} · {} × {}",
                    if display.primary {
                        "主显示器 · "
                    } else {
                        ""
                    },
                    display.width,
                    display.height
                )
                .into(),
                preset: selected
                    .map(|index| rt.presets[index].preset.name.as_str())
                    .unwrap_or("原始显示")
                    .into(),
                preset_index: selected.map_or(-1, |index| index as i32),
                foreground: foreground.into(),
                ready: screen.ready(),
            }
        })
        .collect();
    ui.set_display_cards(model(screen_rows));
}

fn install_desktop(ui: &AppWindow, runtime: &Rc<RefCell<Runtime>>) {
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_apply_desktop(move |display, index| {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        if let Ok(index) = usize::try_from(index)
            && index < rt.presets.len()
        {
            update_desktop(&ui, &mut rt, display.as_str(), Some(index));
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_restore_display(move |display| {
        let Some(ui) = weak.upgrade() else { return };
        update_desktop(&ui, &mut rt.borrow_mut(), display.as_str(), None);
    });
}

fn update_desktop(ui: &AppWindow, rt: &mut Runtime, display: &str, index: Option<usize>) {
    match rt.set_desktop(display, index) {
        Ok(()) => {
            fill_desktop(ui, rt);
            let title = if index.is_some() {
                "已设置桌面预设"
            } else {
                "已清除桌面预设"
            };
            ui.invoke_notify(InfoSeverity::Success, title.into(), "".into());
        }
        Err(error) => {
            ui.invoke_notify(
                InfoSeverity::Error,
                "显示器设置失败".into(),
                error.to_string().into(),
            );
        }
    }
}

fn fill_bindings(ui: &AppWindow, rt: &Runtime) {
    ui.set_bindings(model(
        rt.presets
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| {
                entry.preset.bindings.iter().map(move |exe| BindingDisplay {
                    index: index as i32,
                    executable: exe.as_str().into(),
                    icon: rt
                        .application_icons
                        .get(&exe.to_ascii_lowercase())
                        .cloned()
                        .unwrap_or_default(),
                })
            })
            .collect(),
    ));
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
}

fn push_post_process(ui: &AppWindow, params: PostProcessParams) {
    ui.set_hdr_toning(params.hdr_toning);
    ui.set_bloom(params.bloom);
    ui.set_saturation(params.saturation);
    ui.set_clarity(params.clarity);
    ui.set_vibrance(params.vibrance);
}

fn read_post_process(ui: &AppWindow) -> PostProcessParams {
    PostProcessParams {
        hdr_toning: ui.get_hdr_toning(),
        bloom: ui.get_bloom(),
        saturation: ui.get_saturation(),
        clarity: ui.get_clarity(),
        vibrance: ui.get_vibrance(),
    }
}

fn read_params(ui: &AppWindow, params: &mut FilterParams) {
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
}
