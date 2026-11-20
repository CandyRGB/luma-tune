#[path = "../src/calendar_view.rs"]
mod calendar_view;

use lumatune_win::{accent::light_accent_text_colors, calendar::today};
use slint::{Color, ComponentHandle};

slint::slint! {
    import { CalendarDateText, CalendarDatePickerExample, CalendarViewBehavior, CalendarViewCell, CalendarViewDayName, CalendarViewDisplayMode, CalendarViewOptions, CalendarViewPage, CalendarViewSelectionChange, CalendarViewSelectionMode } from "../../examples/winui3/calendar_date_picker.slint";
    export { CalendarDateText, CalendarDatePickerExample, CalendarViewBehavior, CalendarViewCell, CalendarViewDayName, CalendarViewDisplayMode, CalendarViewOptions, CalendarViewPage, CalendarViewSelectionChange, CalendarViewSelectionMode }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = CalendarDatePickerExample::new()?;
    calendar_view::install(ui.global::<CalendarViewBehavior>());
    ui.set_today(today());
    if let Ok(colors) = light_accent_text_colors() {
        let [red, green, blue] = colors.tertiary;
        ui.set_accent(Color::from_rgb_u8(red, green, blue));
        let [red, green, blue] = colors.primary;
        ui.set_selection_ink(Color::from_rgb_u8(red, green, blue));
    }
    ui.run()
}
