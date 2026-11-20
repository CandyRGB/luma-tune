use lumatune_win::accent::light_accent_text_colors;
use slint::Color;

slint::slint! {
    import { SliderExample } from "../../examples/winui3/slider.slint";
    export { SliderExample }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = SliderExample::new()?;
    if let Ok(colors) = light_accent_text_colors() {
        let [red, green, blue] = colors.tertiary;
        ui.set_accent(Color::from_rgb_u8(red, green, blue));
    }
    ui.run()
}
