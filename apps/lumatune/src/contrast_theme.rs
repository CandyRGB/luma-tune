use std::time::Duration;

use lumatune_win::contrast::{self, ContrastColors};
use slint::{Color, ComponentHandle, Timer, TimerMode};

use crate::{AppWindow, ContrastTheme, SystemContrastColors};

pub fn watch(ui: &AppWindow) -> Timer {
    let mut previous = contrast::current().ok().flatten();
    apply(ui, previous);

    let timer = Timer::default();
    let weak = ui.as_weak();
    timer.start(TimerMode::Repeated, Duration::from_millis(500), move || {
        let Some(ui) = weak.upgrade() else { return };
        let Ok(current) = contrast::current() else {
            return;
        };
        if previous != current {
            apply(&ui, current);
            previous = current;
        }
    });
    timer
}

fn apply(ui: &AppWindow, current: Option<ContrastColors>) {
    fn color([red, green, blue]: [u8; 3]) -> Color {
        Color::from_rgb_u8(red, green, blue)
    }

    ui.global::<ContrastTheme>().set_colors(current.map_or_else(
        SystemContrastColors::default,
        |colors| SystemContrastColors {
            active: true,
            window: color(colors.window),
            window_text: color(colors.window_text),
            button_face: color(colors.button_face),
            button_text: color(colors.button_text),
            disabled_text: color(colors.disabled_text),
            highlight: color(colors.highlight),
            highlight_text: color(colors.highlight_text),
        },
    ));
}
