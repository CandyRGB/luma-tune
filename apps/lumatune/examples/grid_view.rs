#[path = "../src/grid_view.rs"]
mod grid_view;

use std::{path::Path, rc::Rc};

use lumatune_win::accent::light_accent_text_colors;
use slint::{Color, ComponentHandle, Image, ModelRc, VecModel};

slint::slint! {
    import { GridViewBehavior, GridViewExample, GridViewItem, GridViewSelectionMode } from "../../examples/winui3/grid_view.slint";
    export { GridViewBehavior, GridViewExample, GridViewItem, GridViewSelectionMode }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = GridViewExample::new()?;
    grid_view::install(ui.global::<GridViewBehavior>());
    if let Ok(colors) = light_accent_text_colors() {
        let [red, green, blue] = colors.tertiary;
        ui.set_accent(Color::from_rgb_u8(red, green, blue));
    }
    let assets = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.refs/WinUI/WinUI-Gallery/WinUIGallery/Assets/SampleMedia");
    let photos = Rc::new(
        (1..=8)
            .map(|index| GridViewItem {
                id: format!("photo-{index}").into(),
                title: format!("Item {index}").into(),
                description: format!("Landscape photograph {index}").into(),
                image: Image::load_from_path(&assets.join(format!("LandscapeImage{index}.jpg")))
                    .unwrap_or_default(),
                disabled: index == 8,
            })
            .collect::<Vec<_>>(),
    );
    ui.set_items(ModelRc::new(VecModel::from(photos.as_ref().clone())));

    let weak = ui.as_weak();
    ui.on_load_sample(move |large| {
        if let Some(ui) = weak.upgrade() {
            let items = if large {
                (1..=10_000)
                    .map(|index| GridViewItem {
                        id: format!("tile-{index}").into(),
                        title: format!("Tile {index}").into(),
                        ..Default::default()
                    })
                    .collect()
            } else {
                photos.as_ref().clone()
            };
            ui.set_items(ModelRc::new(VecModel::from(items)));
        }
    });
    ui.run()
}
