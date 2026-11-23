use slint::ComponentHandle;

#[path = "../src/relative_panel.rs"]
mod relative_panel;

slint::slint! {
    import {
        ItemsRepeaterGeometry, ItemsRepeaterPlacement, ItemsRepeaterSize,
        RelativePanelAlignment, RelativePanelAxis, RelativePanelBehavior, RelativePanelChild,
        RelativePanelExample, RelativePanelInsets, RelativePanelIssue, RelativePanelLayout
    } from "../../examples/winui3/relative_panel.slint";
    export {
        ItemsRepeaterGeometry, ItemsRepeaterPlacement, ItemsRepeaterSize,
        RelativePanelAlignment, RelativePanelAxis, RelativePanelBehavior, RelativePanelChild,
        RelativePanelExample, RelativePanelInsets, RelativePanelIssue, RelativePanelLayout
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = RelativePanelExample::new()?;
    relative_panel::install(ui.global::<RelativePanelBehavior>());
    ui.run()
}
