slint::slint! {
    import { ExpanderExample } from "../../examples/winui3/expander.slint";
    export { ExpanderExample }
}

fn main() -> Result<(), slint::PlatformError> {
    ExpanderExample::new()?.run()
}
