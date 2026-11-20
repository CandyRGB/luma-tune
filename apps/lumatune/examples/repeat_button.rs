slint::slint! {
    import { RepeatButtonExample } from "../../examples/winui3/repeat_button.slint";
    export { RepeatButtonExample }
}

fn main() -> Result<(), slint::PlatformError> {
    RepeatButtonExample::new()?.run()
}
