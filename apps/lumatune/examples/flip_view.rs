#[path = "../src/flip_view.rs"]
mod flip_view;

use std::{path::Path, rc::Rc};

use slint::{ComponentHandle, Image, Model, ModelRc, VecModel};

slint::slint! {
    import { FlipViewBehavior, FlipViewExample, FlipViewItem } from "../../examples/winui3/flip_view.slint";
    export { FlipViewBehavior, FlipViewExample, FlipViewItem }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = FlipViewExample::new()?;
    flip_view::install(ui.global::<FlipViewBehavior>());
    let assets = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.refs/WinUI/WinUI-Gallery/WinUIGallery/Assets/SampleMedia");
    let original = Rc::new(
        [
            ("cliff", "Cliff"),
            ("grapes", "Grapes"),
            ("rainier", "Rainier"),
            ("sunset", "Sunset"),
            ("valley", "Valley"),
        ]
        .into_iter()
        .map(|(id, title)| FlipViewItem {
            id: id.into(),
            title: title.into(),
            image: Image::load_from_path(&assets.join(format!("{id}.jpg"))).unwrap_or_default(),
        })
        .collect::<Vec<_>>(),
    );
    ui.set_items(ModelRc::new(VecModel::from(original.as_ref().clone())));

    let weak = ui.as_weak();
    ui.on_remove_current(move || {
        if let Some(ui) = weak.upgrade() {
            let items: Vec<_> = ui
                .get_items()
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    (index as i32 != ui.get_selected_index()).then_some(item)
                })
                .collect();
            ui.set_items(ModelRc::new(VecModel::from(items)));
        }
    });
    let weak = ui.as_weak();
    ui.on_move_current_first(move || {
        if let Some(ui) = weak.upgrade() {
            let mut items: Vec<_> = ui.get_items().iter().collect();
            let index = ui.get_selected_index() as usize;
            if index < items.len() {
                items[..=index].rotate_right(1);
                ui.set_items(ModelRc::new(VecModel::from(items)));
            }
        }
    });
    let weak = ui.as_weak();
    ui.on_restore_items(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_items(ModelRc::new(VecModel::from(original.as_ref().clone())));
        }
    });

    ui.run()
}
