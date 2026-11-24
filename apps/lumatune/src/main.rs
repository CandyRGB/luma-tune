use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use lumatune_core::{FilterParams, Preset, Ramp};
use lumatune_win::presets::StoredPreset;
use lumatune_win::settings::{self, ThemeMode};
use lumatune_win::{Session, display, file_picker, foreground, presets, state};
use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};

mod contrast_theme;

slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/slint_winui_behaviors.rs"));

struct Draft {
    index: Option<usize>,
    params: FilterParams,
}

struct Runtime {
    presets: Vec<StoredPreset>,
    desktop: Option<usize>,
    active: Option<usize>,
    foreground: String,
    draft: Option<Draft>,
    comparing: bool,
    dirty: bool,
}

impl Runtime {
    fn current_ramp(&self) -> Option<Ramp> {
        if let Some(index) = self.active {
            return Some(self.presets[index].preset.params.to_ramp());
        }
        if self.comparing {
            return None;
        }
        let params = match &self.draft {
            Some(draft) => &draft.params,
            None => &self.presets.get(self.desktop?)?.preset.params,
        };
        (!params.is_neutral()).then(|| params.to_ramp())
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

    fn save_draft(&mut self, name: &str, cover: &CoverChoice) -> io::Result<()> {
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
        let old_cover = updated.cover.clone();
        let imported = if cover.changed && !cover.source.is_empty() {
            let name = presets::import_cover(Path::new(cover.source.as_str()))?;
            let readable =
                presets::cover_path(&name).is_some_and(|path| Image::load_from_path(&path).is_ok());
            if !readable {
                self.forget_cover(Some(&name));
                return Err(invalid("无法读取所选主图"));
            }
            Some(name)
        } else {
            None
        };
        if cover.changed {
            updated.cover = imported.clone();
        }
        let saved = if let Some(index) = draft.index {
            self.presets[index].save(updated).map(|()| index)
        } else {
            presets::create(updated).map(|entry| {
                let index = self.presets.len();
                self.presets.push(entry);
                index
            })
        };
        match saved {
            Ok(index) => {
                self.desktop = Some(index);
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

    fn delete_draft(&mut self) -> io::Result<()> {
        let index = self
            .draft
            .as_ref()
            .and_then(|draft| draft.index)
            .ok_or_else(|| invalid("没有可删除的预设"))?;
        self.presets[index].delete()?;
        let removed = self.presets.remove(index);
        self.draft = None;
        self.desktop = self.desktop.and_then(|current| match current.cmp(&index) {
            std::cmp::Ordering::Less => Some(current),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(current - 1),
        });
        self.refresh_active();
        self.dirty = true;
        self.forget_cover(removed.preset.cover.as_deref());
        Ok(())
    }

    fn add_binding(&mut self, name: &str, index: i32) -> io::Result<()> {
        let name = name.trim();
        if !name.to_ascii_lowercase().ends_with(".exe")
            || name.len() <= 4
            || name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
            || name.chars().any(char::is_control)
            || name.eq_ignore_ascii_case("lumatune.exe")
        {
            return Err(invalid("请输入目标程序的 .exe 文件名"));
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
        self.refresh_active();
        Ok(())
    }

    fn remove_binding(&mut self, name: &str, index: i32) -> io::Result<()> {
        let entry = usize::try_from(index)
            .ok()
            .and_then(|i| self.presets.get_mut(i))
            .ok_or_else(|| invalid("找不到该绑定"))?;
        let mut updated = entry.preset.clone();
        updated.unbind(name);
        entry.save(updated)?;
        self.refresh_active();
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn main() -> Result<(), slint::PlatformError> {
    display::set_dpi_awareness();
    let displays = display::enumerate();
    let healed = state::heal(&displays);
    let target = display::primary(&displays);
    let ui = AppWindow::new()?;
    breadcrumb_bar::install(ui.global::<BreadcrumbBarBehavior>());
    combo_box::install(ui.global::<ComboBoxBehavior>());
    navigation_view::install(ui.global::<NavigationViewBehavior>(), &ui);

    let prefs = RefCell::new(settings::load());
    ui.set_theme_mode(prefs.borrow().theme.index());
    let _contrast_watcher = contrast_theme::watch(&ui);
    let mut session = target.and_then(Session::open);
    let mut notes = Vec::new();
    if healed {
        notes.push("上次未正常退出，已恢复显示状态");
    }
    match target {
        Some(d) => {
            ui.set_display_text(format!("{} · {} × {}", d.device, d.width, d.height).into());
            let mode = if d.hdr() {
                "HDR"
            } else if session.is_some() {
                "SDR · 调校可用"
            } else {
                "SDR · 调校不可用"
            };
            ui.set_display_mode(mode.into());
            if d.hdr() {
                notes.push("该显示器已开启 HDR，滤镜可能无法生效");
            } else if d.looks_virtual() {
                notes.push("检测到虚拟显示适配器，调校效果可能不同");
            }
        }
        None => {
            ui.set_display_text("未找到可用显示器".into());
            ui.set_display_mode("不可用".into());
        }
    }
    ui.set_status(notes.join("；").into());

    let runtime = Rc::new(RefCell::new(Runtime {
        presets: presets::list(),
        desktop: None,
        active: None,
        foreground: String::new(),
        draft: None,
        comparing: false,
        dirty: false,
    }));
    fill_library(&ui, &runtime.borrow());
    install_editor(&ui, &runtime);
    install_bindings(&ui, &runtime);

    let weak = ui.as_weak();
    ui.on_pick_theme(move |index| {
        let Some(ui) = weak.upgrade() else { return };
        let mut prefs = prefs.borrow_mut();
        prefs.theme = ThemeMode::from_index(index);
        ui.set_theme_mode(prefs.theme.index());
        if let Err(error) = settings::save(&prefs) {
            ui.set_status(format!("主题保存失败：{error}").into());
        }
    });

    let writer = Timer::default();
    let rt = runtime.clone();
    writer.start(TimerMode::Repeated, Duration::from_millis(30), move || {
        let mut rt = rt.borrow_mut();
        if !std::mem::take(&mut rt.dirty) {
            return;
        }
        if let Some(session) = session.as_mut() {
            match rt.current_ramp() {
                Some(ramp) => {
                    session.apply(&ramp);
                }
                None => session.restore(),
            }
        }
    });

    let watcher = Timer::default();
    let weak = ui.as_weak();
    watcher.start(TimerMode::Repeated, Duration::from_millis(400), move || {
        let Some(ui) = weak.upgrade() else { return };
        let Some(fg) = foreground::current() else {
            return;
        };
        let mut rt = runtime.borrow_mut();
        if !rt.foreground.eq_ignore_ascii_case(&fg.exe) {
            rt.foreground = fg.exe;
            rt.refresh_active();
            fill_bindings(&ui, &rt);
        }
    });

    let result = ui.run();
    // The timer owns the display session. Dropping it restores the original table.
    drop(writer);
    drop(watcher);
    result
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
        rt.draft = Some(Draft { index, params });
        rt.dirty = true;
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_parameters_edited(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        if let Some(draft) = rt.draft.as_mut() {
            read_params(&ui, &mut draft.params);
            rt.dirty = true;
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
            draft.params = params;
            rt.dirty = true;
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_close_editor(move || {
        let mut rt = rt.borrow_mut();
        rt.draft = None;
        rt.comparing = false;
        rt.dirty = true;
        if let Some(ui) = weak.upgrade() {
            ui.set_comparing(false);
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_toggle_compare(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        rt.comparing = !rt.comparing;
        rt.dirty = true;
        ui.set_comparing(rt.comparing);
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_apply_preset(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut rt = rt.borrow_mut();
        rt.desktop = rt.draft.as_ref().and_then(|draft| draft.index);
        rt.dirty = true;
        fill_desktop(&ui, &rt);
        if let Some(index) = rt.desktop {
            ui.set_status(format!("已应用「{}」", rt.presets[index].preset.name).into());
        }
    });

    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_save_preset(move |name, cover| {
        let Some(ui) = weak.upgrade() else {
            return false;
        };
        let mut rt = rt.borrow_mut();
        match rt.save_draft(name.as_str(), &cover) {
            Ok(()) => {
                fill_library(&ui, &rt);
                ui.set_status(format!("已保存「{}」", name.trim()).into());
                true
            }
            Err(error) => {
                ui.set_status(format!("保存失败：{error}").into());
                false
            }
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_delete_preset(move || {
        let Some(ui) = weak.upgrade() else {
            return false;
        };
        let mut rt = rt.borrow_mut();
        match rt.delete_draft() {
            Ok(()) => {
                fill_library(&ui, &rt);
                ui.set_status("已删除预设".into());
                true
            }
            Err(error) => {
                ui.set_status(format!("删除失败：{error}").into());
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
            Err(error) => ui.set_status(format!("收藏保存失败：{error}").into()),
        }
    });
    let weak = ui.as_weak();
    ui.on_browse_cover(move || {
        let Some(ui) = weak.upgrade() else {
            return CoverChoice::default();
        };
        ui.set_status("".into());
        match file_picker::cover() {
            Ok(Some(path)) => match Image::load_from_path(&path) {
                Ok(artwork) => CoverChoice {
                    artwork,
                    source: path.to_string_lossy().as_ref().into(),
                    changed: true,
                },
                Err(error) => {
                    ui.set_status(format!("无法打开图片：{error}").into());
                    CoverChoice::default()
                }
            },
            Ok(None) => CoverChoice::default(),
            Err(error) => {
                ui.set_status(format!("无法选择图片：{error}").into());
                CoverChoice::default()
            }
        }
    });
}

fn install_bindings(ui: &AppWindow, runtime: &Rc<RefCell<Runtime>>) {
    let weak = ui.as_weak();
    ui.on_browse_application(move || {
        let Some(ui) = weak.upgrade() else {
            return SharedString::default();
        };
        match file_picker::executable() {
            Ok(Some(path)) => {
                ui.set_status("".into());
                path.file_name()
                    .map(|name| name.to_string_lossy().as_ref().into())
                    .unwrap_or_default()
            }
            Ok(None) => SharedString::default(),
            Err(error) => {
                ui.set_status(format!("无法选择程序：{error}").into());
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
                ui.set_status("已保存应用绑定".into());
                true
            }
            Err(error) => {
                ui.set_status(format!("绑定失败：{error}").into());
                false
            }
        }
    });
    let weak = ui.as_weak();
    let rt = runtime.clone();
    ui.on_remove_binding(move |exe, index| {
        let Some(ui) = weak.upgrade() else {
            return false;
        };
        let mut rt = rt.borrow_mut();
        match rt.remove_binding(exe.as_str(), index) {
            Ok(()) => {
                fill_bindings(&ui, &rt);
                ui.set_status("已解除应用绑定".into());
                true
            }
            Err(error) => {
                ui.set_status(format!("解除失败：{error}").into());
                false
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
    ui.set_desktop_preset(
        rt.desktop
            .map(|i| rt.presets[i].preset.name.as_str())
            .unwrap_or("未应用预设")
            .into(),
    );
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
                    preset: entry.preset.name.as_str().into(),
                    active: Some(index) == rt.active && exe.eq_ignore_ascii_case(&rt.foreground),
                })
            })
            .collect(),
    ));
    ui.set_active_binding(
        rt.active
            .map(|index| {
                format!(
                    "{} 正在使用「{}」。",
                    rt.foreground, rt.presets[index].preset.name
                )
                .into()
            })
            .unwrap_or_default(),
    );
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
