use std::path::Path;

use slint::{ComponentHandle, Image, ModelRc, VecModel};

#[path = "../src/scroll_snap.rs"]
mod scroll_snap;

slint::slint! {
    import { ScrollPhoto, ScrollPhotoStrip, ScrollViewExample } from "../../examples/winui3/scroll_view.slint";
    import {
        ScrollSnapAlignment, ScrollSnapBehavior, ScrollSnapIntent, ScrollSnapPoint, ScrollSnapPoints,
        ScrollSnapRange, ScrollSnapRequest
    } from "ui/components/scrolling/scroll_snap.slint";
    export { ScrollPhoto, ScrollPhotoStrip, ScrollViewExample }
    export { ScrollSnapAlignment, ScrollSnapBehavior, ScrollSnapIntent, ScrollSnapPoint, ScrollSnapPoints, ScrollSnapRange, ScrollSnapRequest }
}

fn load_strip<'a>(
    media: &Path,
    files: impl Iterator<Item = (&'a str, String)>,
) -> ScrollPhotoStrip {
    let photos = files
        .map(|(name, file)| {
            let image = Image::load_from_path(&media.join(file)).unwrap_or_default();
            let size = image.size();
            let height = if size.width > 0 {
                (200.0 * size.height as f32 / size.width as f32).round() * 2.0
            } else {
                240.0
            };
            ScrollPhoto {
                name: name.into(),
                image,
                height,
            }
        })
        .collect::<Vec<_>>();
    ScrollPhotoStrip {
        height: photos.iter().map(|photo| photo.height).sum(),
        photos: ModelRc::new(VecModel::from(photos)),
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let window = ScrollViewExample::new()?;
    scroll_snap::install(window.global::<ScrollSnapBehavior>());
    if std::env::var("SLINT_BACKEND").is_ok_and(|backend| backend.contains("software")) {
        window.set_transform_content(false);
    }
    let media = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.refs/WinUI/WinUI-Gallery/WinUIGallery/Assets/SampleMedia");
    window.set_cliff(Image::load_from_path(&media.join("cliff.jpg")).unwrap_or_default());
    let landscape = [
        "leaves", "carousel", "bicycles", "pond", "marina", "beach", "rampart", "mountain",
    ];
    let velocity = ["grapes", "rainier", "sunset", "treetops", "valley", "cliff"];
    window.set_animation_strip(load_strip(
        &media,
        landscape
            .into_iter()
            .enumerate()
            .map(|(index, name)| (name, format!("LandscapeImage{}.jpg", index + 1))),
    ));
    window.set_velocity_strip(load_strip(
        &media,
        velocity
            .into_iter()
            .map(|name| (name, format!("{name}.jpg"))),
    ));
    window.run()
}
