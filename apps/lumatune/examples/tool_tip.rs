use std::path::Path;

use lumatune_win::tool_tip::timing;
use slint::Image;

slint::slint! {
    import { ToolTipExample } from "../../examples/winui3/tool_tip.slint";
    export { ToolTipExample }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = ToolTipExample::new()?;
    let timing = timing();
    ui.set_system_show_delay(timing.show_delay.as_millis() as i64);
    ui.set_system_show_duration(timing.show_duration.as_millis() as i64);
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.refs/WinUI/WinUI-Gallery/WinUIGallery/Assets/SampleMedia/cliff.jpg");
    if let Ok(image) = Image::load_from_path(&path) {
        ui.set_gallery_photo(image);
    }
    ui.run()
}
