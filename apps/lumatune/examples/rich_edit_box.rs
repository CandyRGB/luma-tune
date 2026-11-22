use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::num::NonZeroIsize;
use std::rc::Rc;
use std::time::Duration;

use lumatune_win::{
    accent::light_accent_text_colors,
    rich_edit::{Edit, EditorRequest, Effect, Placement, RichEdit},
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use slint::{Color, ComponentHandle, Timer, TimerMode};

slint::slint! {
    import { RichEditBoxExample, RichEditCommand, RichEditGeometry, RichEditState }
        from "../../examples/winui3/rich_edit_box.slint";
    export { RichEditBoxExample, RichEditCommand, RichEditGeometry, RichEditState }
}

struct MountedEditor {
    editor: RichEdit,
    geometry: Option<RichEditGeometry>,
    scale: f32,
    has_text: bool,
}

impl MountedEditor {
    fn new(editor: RichEdit) -> Self {
        editor.set_default_font("Segoe UI Variable Text", 14.0);
        Self {
            editor,
            geometry: None,
            scale: 0.0,
            has_text: false,
        }
    }

    fn update(
        &mut self,
        geometry: RichEditGeometry,
        scale: f32,
    ) -> Result<RichEditState, Box<dyn Error>> {
        let previous = self.geometry.as_ref();
        if previous != Some(&geometry) || self.scale != scale {
            let px = |logical: f32| (logical * scale).round() as i32;
            if self.scale != scale {
                self.editor.set_scale(scale)?;
            }
            let placement_changed = previous.is_none_or(|old| {
                (old.x, old.y, old.width, old.height, old.visible)
                    != (
                        geometry.x,
                        geometry.y,
                        geometry.width,
                        geometry.height,
                        geometry.visible,
                    )
            });
            if placement_changed || self.scale != scale {
                self.editor.place(
                    Placement {
                        left: px(geometry.x),
                        top: px(geometry.y),
                        width: px(geometry.x + geometry.width) - px(geometry.x),
                        height: px(geometry.y + geometry.height) - px(geometry.y),
                        padding_x: px(8.0),
                        padding_y: px(4.0),
                    },
                    geometry.visible,
                )?;
            }
            if previous.is_none_or(|old| old.fill != geometry.fill) {
                self.editor.set_background(rgb(geometry.fill));
            }
            if previous.is_none_or(|old| old.enabled != geometry.enabled) {
                self.editor.set_enabled(geometry.enabled);
            }
            if previous.is_none_or(|old| old.read_only != geometry.read_only) {
                self.editor.set_read_only(geometry.read_only);
            }
            self.geometry = Some(geometry);
            self.scale = scale;
        }
        if self.editor.take_text_changed() {
            self.has_text = self.editor.has_text();
        }
        let selection = self.editor.selection();
        Ok(RichEditState {
            has_focus: self.editor.has_focus(),
            has_text: self.has_text,
            has_selection: selection.start != selection.end,
            can_undo: self.editor.can_undo(),
            can_redo: self.editor.can_redo(),
            bold: self.editor.effect_active(Effect::Bold),
            italic: self.editor.effect_active(Effect::Italic),
            underlined: self.editor.effect_active(Effect::Underline),
        })
    }
}

fn rgb(color: Color) -> [u8; 3] {
    [color.red(), color.green(), color.blue()]
}

fn command(editor: &RichEdit, command: RichEditCommand) {
    let action = match command {
        RichEditCommand::Focus => {
            editor.focus();
            return;
        }
        RichEditCommand::Cut => Edit::Cut,
        RichEditCommand::Copy => Edit::Copy,
        RichEditCommand::Paste => Edit::Paste,
        RichEditCommand::SelectAll => Edit::SelectAll,
        RichEditCommand::Undo => Edit::Undo,
        RichEditCommand::Redo => Edit::Redo,
        RichEditCommand::Bold => Edit::Toggle(Effect::Bold),
        RichEditCommand::Italic => Edit::Toggle(Effect::Italic),
        RichEditCommand::Underline => Edit::Toggle(Effect::Underline),
    };
    editor.execute(action);
    editor.focus();
}

fn open_file(editor: &RichEdit) -> io::Result<Option<String>> {
    let Some(path) = editor.pick_rtf_file(false)? else {
        return Ok(None);
    };
    let bytes = fs::read(&path)?;
    editor.import_rtf(&bytes)?;
    editor.focus();
    Ok(Some(format!("Opened {}", path.display())))
}

fn save_file(editor: &RichEdit) -> io::Result<Option<String>> {
    let Some(path) = editor.pick_rtf_file(true)? else {
        return Ok(None);
    };
    let output = editor.export_rtf()?;
    let directory = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
    // Swap the completed file into place so a partial write preserves the existing document.
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    temporary.write_all(&output)?;
    temporary.as_file().sync_all()?;
    temporary.persist(&path).map_err(|error| error.error)?;
    editor.focus();
    Ok(Some(format!("Saved {}", path.display())))
}

fn show_status(ui: &RichEditBoxExample, result: io::Result<Option<String>>) {
    match result {
        Ok(Some(message)) => ui.set_status(message.into()),
        Err(error) => ui.set_status(error.to_string().into()),
        Ok(None) => {}
    }
}

fn tick(ui: &RichEditBoxExample, slot: &RefCell<MountedEditor>, simple: bool) {
    let Ok(mut host) = slot.try_borrow_mut() else {
        return;
    };
    let geometry = if simple {
        ui.get_simple_geometry()
    } else {
        ui.get_custom_geometry()
    };
    match host.update(geometry, ui.window().scale_factor()) {
        Ok(state) if simple => {
            if ui.get_simple_state() != state {
                ui.set_simple_state(state);
            }
        }
        Ok(state) => {
            if ui.get_custom_state() != state {
                ui.set_custom_state(state);
            }
        }
        Err(error) => ui.set_status(error.to_string().into()),
    }
    match host.editor.take_request() {
        Some(EditorRequest::ContextMenu { x, y }) => {
            if let Err(error) = host.editor.show_context_menu(x, y) {
                ui.set_status(error.to_string().into());
            }
        }
        Some(EditorRequest::Traverse { backward }) => {
            host.editor.yield_focus();
            drop(host);
            ui.invoke_traverse_from_editor(simple, backward);
        }
        None => {}
    }
}

struct MountedPair {
    simple: Rc<RefCell<MountedEditor>>,
    custom: Rc<RefCell<MountedEditor>>,
}

fn mount(ui: &RichEditBoxExample, parent: NonZeroIsize) -> Result<MountedPair, Box<dyn Error>> {
    let simple = Rc::new(RefCell::new(MountedEditor::new(RichEdit::new(parent)?)));
    let custom = Rc::new(RefCell::new(MountedEditor::new(RichEdit::new(parent)?)));
    simple
        .borrow()
        .editor
        .set_accessible_name("Simple text editor")?;
    custom
        .borrow()
        .editor
        .set_accessible_name("Custom rich text editor")?;
    if std::env::args_os().any(|arg| arg == "--sample-text") {
        custom.borrow().editor.set_text("Apply a format to a selection, then save the document as RTF.\nThe editor preserves Unicode, including 中文 and 😀.")?;
    }

    let host = simple.clone();
    ui.on_simple_command(move |action| {
        if let Ok(host) = host.try_borrow() {
            command(&host.editor, action);
        }
    });
    let host = custom.clone();
    ui.on_custom_command(move |action| {
        if let Ok(host) = host.try_borrow() {
            command(&host.editor, action);
        }
    });
    let host = custom.clone();
    ui.on_apply_color(move |color, automatic| {
        if let Ok(host) = host.try_borrow() {
            host.editor
                .set_selection_color((!automatic).then(|| rgb(color)));
            host.editor.focus();
        }
    });
    let host = custom.clone();
    let weak = ui.as_weak();
    ui.on_open_document(move || {
        if let (Some(ui), Ok(host)) = (weak.upgrade(), host.try_borrow_mut()) {
            show_status(&ui, open_file(&host.editor));
        }
    });
    let host = custom.clone();
    let weak = ui.as_weak();
    ui.on_save_document(move || {
        if let (Some(ui), Ok(host)) = (weak.upgrade(), host.try_borrow_mut()) {
            show_status(&ui, save_file(&host.editor));
        }
    });
    let host = custom.clone();
    let weak = ui.as_weak();
    ui.on_find_next(move |needle| {
        if let (Some(ui), Ok(host)) = (weak.upgrade(), host.try_borrow()) {
            let editor = &host.editor;
            let selected = editor.selection();
            if let Some(range) = editor
                .find_from(&needle, selected.end)
                .or_else(|| editor.find_from(&needle, 0))
            {
                editor.set_selection(range);
                editor.reveal_selection();
                editor.focus();
                ui.set_status("".into());
            } else {
                ui.set_status("No matches found".into());
            }
        }
    });

    Ok(MountedPair { simple, custom })
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = RichEditBoxExample::new()?;
    if let Ok(accent) = light_accent_text_colors() {
        let [red, green, blue] = accent.tertiary;
        ui.set_accent(Color::from_rgb_u8(red, green, blue));
    }
    ui.show()?;
    let weak = ui.as_weak();
    let mounted: RefCell<Option<MountedPair>> = RefCell::new(None);
    let remaining_attempts = Cell::new(20u32);
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(25), move || {
        let (Some(ui), Ok(mut mounted)) = (weak.upgrade(), mounted.try_borrow_mut()) else {
            return;
        };
        if mounted.is_none() {
            let Some(remaining) = remaining_attempts.get().checked_sub(1) else {
                return;
            };
            remaining_attempts.set(remaining);
            // A winit window exposes its HWND after processing the first event-loop turn.
            let window = ui.window().window_handle();
            let Ok(handle) = window.window_handle() else {
                if remaining == 0 {
                    ui.set_status(
                        "The window backend did not provide a native Windows handle.".into(),
                    );
                }
                return;
            };
            let RawWindowHandle::Win32(handle) = handle.as_raw() else {
                ui.set_status("RichEditBox requires a Windows host window.".into());
                remaining_attempts.set(0);
                return;
            };
            match mount(&ui, handle.hwnd) {
                Ok(pair) => *mounted = Some(pair),
                Err(error) => {
                    ui.set_status(error.to_string().into());
                    remaining_attempts.set(0);
                    return;
                }
            }
        }
        if let Some(pair) = &*mounted {
            tick(&ui, &pair.simple, true);
            tick(&ui, &pair.custom, false);
        }
    });
    slint::run_event_loop()?;
    Ok(())
}
