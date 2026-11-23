use slint::ComponentHandle;

slint::slint! {
    import { BorderCanvasExample } from "../../examples/winui3/border_canvas.slint";
    export { BorderCanvasExample }
}

fn main() -> Result<(), slint::PlatformError> {
    BorderCanvasExample::new()?.run()
}
