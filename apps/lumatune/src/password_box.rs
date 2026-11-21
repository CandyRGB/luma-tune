use crate::{PasswordBoxBehavior, PasswordEdit, PasswordMask};
use slint::SharedString;

pub fn install(behavior: PasswordBoxBehavior<'_>) {
    behavior.set_allow_peek(lumatune_win::password_reveal::allowed());
    behavior.on_mask_glyph(|glyph| glyph_char(&glyph).into());
    behavior.on_mask_text(mask_text);
    behavior.on_limit_edit(limit_edit);
}

fn glyph_char(glyph: &str) -> &str {
    let mut chars = glyph.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) if ch.len_utf16() == 1 && !ch.is_control() => glyph,
        _ => "●",
    }
}

fn mask_text(
    password: SharedString,
    glyph: SharedString,
    anchor: i32,
    cursor: i32,
) -> PasswordMask {
    let lower = count_before(&password, anchor.min(cursor));
    let upper = count_before(&password, anchor.max(cursor));
    let glyph = glyph_char(&glyph);
    let count = password.chars().count();
    PasswordMask {
        display: glyph.repeat(count).into(),
        selected: glyph.repeat(upper - lower).into(),
        selection_start: lower as i32,
    }
}

fn count_before(text: &str, offset: i32) -> usize {
    text.char_indices()
        .take_while(|&(index, _)| index < offset.max(0) as usize)
        .count()
}

fn limit_edit(
    previous: SharedString,
    draft: SharedString,
    cursor: i32,
    maximum: i32,
) -> PasswordEdit {
    let cursor = draft.floor_char_boundary((cursor.max(0) as usize).min(draft.len()));
    if maximum <= 0 {
        return PasswordEdit {
            text: draft,
            cursor: cursor as i32,
        };
    }

    // A programmatic value can be longer than MaxLength. Allow subsequent edits to shorten it.
    let budget = (maximum as usize).max(previous.encode_utf16().count());
    if draft.encode_utf16().count() <= budget {
        return PasswordEdit {
            text: draft,
            cursor: cursor as i32,
        };
    }

    let (head, tail) = draft.split_at(cursor);
    let unchanged_head = previous.strip_suffix(tail).unwrap_or(&previous);
    let prefix_bytes: usize = head
        .chars()
        .zip(unchanged_head.chars())
        .take_while(|(current, old)| current == old)
        .map(|(ch, _)| ch.len_utf8())
        .sum();
    let (prefix, inserted) = head.split_at(prefix_bytes);
    let mut remaining =
        budget.saturating_sub(prefix.encode_utf16().count() + tail.encode_utf16().count());
    let inserted_bytes: usize = inserted
        .chars()
        .take_while(|ch| {
            if let Some(left) = remaining.checked_sub(ch.len_utf16()) {
                remaining = left;
                true
            } else {
                false
            }
        })
        .map(char::len_utf8)
        .sum();
    let end = prefix_bytes + inserted_bytes;
    PasswordEdit {
        text: format!("{}{tail}", &head[..end]).into(),
        cursor: end as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::ComponentHandle;
    use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
    use slint::platform::{Clipboard, Platform, PlatformError, WindowAdapter, set_platform};
    use std::{cell::RefCell, rc::Rc};

    mod harness {
        slint::slint! {
            import { PasswordBox, PasswordBoxBehavior, PasswordEdit }
                from "ui/components/text/password_box.slint";
            export { PasswordBoxBehavior, PasswordEdit }
            export component PasswordHarness inherits Window {
                out property <string> result: field.password;
                in property <bool> cancel-paste;
                in property <bool> cancel-change;
                public function paste-at(anchor: int, cursor: int) {
                    field.set-selection-offsets(anchor, cursor);
                    field.paste();
                }
                field := PasswordBox {
                    password: "abz";
                    max-length: 6;
                    paste-requested => { return root.cancel-paste; }
                    before-password-changing(value) => { return root.cancel-change; }
                }
            }
        }
    }

    struct PastePlatform {
        window: Rc<MinimalSoftwareWindow>,
        text: Rc<RefCell<String>>,
    }

    impl Platform for PastePlatform {
        fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
            Ok(self.window.clone())
        }

        fn clipboard_text(&self, _: Clipboard) -> Option<String> {
            Some(self.text.borrow().clone())
        }
    }

    #[test]
    fn limit_preserves_the_suffix_and_whole_utf16_characters() {
        let edit = limit_edit("abz".into(), "ab😀xyz".into(), 8, 6);
        assert_eq!(edit.text, "ab😀xz");
        assert_eq!(edit.cursor, 7);
        let edit = limit_edit("az".into(), "a😀z".into(), 5, 3);
        assert_eq!(edit.text, "az");
        assert_eq!(edit.cursor, 1);
    }

    #[test]
    fn custom_mask_uses_character_counts_for_utf8_selection() {
        let mask = mask_text("a雪😀z".into(), "#".into(), 8, 1);
        assert_eq!(mask.display, "####");
        assert_eq!(mask.selected, "##");
        assert_eq!(mask.selection_start, 1);
        assert_eq!(glyph_char("😀"), "●");
    }

    #[test]
    fn paste_preserves_existing_text_and_obeys_cancellation() {
        let text = Rc::new(RefCell::new("😀xy".to_owned()));
        let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        set_platform(Box::new(PastePlatform { window, text })).unwrap();
        let ui = harness::PasswordHarness::new().unwrap();
        ui.global::<harness::PasswordBoxBehavior>().on_limit_edit(
            |previous, draft, cursor, maximum| {
                let edit = limit_edit(previous, draft, cursor, maximum);
                harness::PasswordEdit {
                    text: edit.text,
                    cursor: edit.cursor,
                }
            },
        );

        ui.set_cancel_paste(true);
        ui.invoke_paste_at(2, 2);
        assert_eq!(ui.get_result(), "abz");
        ui.set_cancel_paste(false);
        ui.set_cancel_change(true);
        ui.invoke_paste_at(2, 2);
        assert_eq!(ui.get_result(), "abz");
        ui.set_cancel_change(false);
        ui.invoke_paste_at(2, 2);
        assert_eq!(ui.get_result(), "ab😀xz");
    }
}
