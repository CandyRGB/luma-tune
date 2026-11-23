use slint::ComponentHandle;

#[path = "../src/stack_panel.rs"]
mod stack_panel;

slint::slint! {
    import {
        StackPanelExample, StackPanelBehavior, StackPanelChild, StackPanelOptions, StackPanelSnapAlignment,
        StackPanelSnapPoints, BorderInsets, ItemsRepeaterGeometry, ItemsRepeaterPlacement, ItemsRepeaterSize
    } from "../../examples/winui3/stack_panel.slint";
    export {
        StackPanelExample, StackPanelBehavior, StackPanelChild, StackPanelOptions, StackPanelSnapAlignment,
        StackPanelSnapPoints, BorderInsets, ItemsRepeaterGeometry, ItemsRepeaterPlacement, ItemsRepeaterSize
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = StackPanelExample::new()?;
    stack_panel::install(ui.global::<StackPanelBehavior>());
    ui.run()
}
