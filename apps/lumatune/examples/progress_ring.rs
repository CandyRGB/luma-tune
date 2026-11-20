slint::slint! {
    import { ProgressRingExample } from "../../examples/winui3/progress_ring.slint";
    export { ProgressRingExample }
}

fn main() -> Result<(), slint::PlatformError> {
    ProgressRingExample::new()?.run()
}
