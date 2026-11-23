use slint::ComponentHandle;

#[path = "../src/grid.rs"]
mod grid;

slint::slint! {
    import {
        GridExample, GridBehavior, GridChild, GridDefinition, GridLayoutResult, GridLengthKind,
        GridMeasureRequest, GridOptions, GridTrackPlacement, ItemsRepeaterGeometry, ItemsRepeaterPlacement,
        ItemsRepeaterSize
    } from "../../examples/winui3/grid.slint";
    export {
        GridExample, GridBehavior, GridChild, GridDefinition, GridLayoutResult, GridLengthKind,
        GridMeasureRequest, GridOptions, GridTrackPlacement, ItemsRepeaterGeometry, ItemsRepeaterPlacement,
        ItemsRepeaterSize
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = GridExample::new()?;
    grid::install(ui.global::<GridBehavior>(), |request| request.preferred);
    ui.run()
}
