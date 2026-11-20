use lumatune_win::accent::light_accent_text_colors;
use slint::{Color, ComponentHandle};

slint::slint! {
    import { HyperlinkButtonExample, HyperlinkButtonTheme, HyperlinkButtonAccentColors } from "../../examples/winui3/hyperlink_button.slint";
    export { HyperlinkButtonExample, HyperlinkButtonTheme, HyperlinkButtonAccentColors }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = HyperlinkButtonExample::new()?;
    if let Some(uri) = std::env::args().nth(1) {
        ui.set_demo_uri(uri.into());
    }
    if let Ok(colors) = light_accent_text_colors() {
        let rgb = |[r, g, b]: [u8; 3]| Color::from_rgb_u8(r, g, b);
        ui.global::<HyperlinkButtonTheme>()
            .set_light_accent_text(HyperlinkButtonAccentColors {
                primary: rgb(colors.primary),
                secondary: rgb(colors.secondary),
                tertiary: rgb(colors.tertiary),
            });
    }
    ui.run()
}
