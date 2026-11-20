slint::slint! {
    import { RatingControlExample } from "../../examples/winui3/rating_control.slint";
    export { RatingControlExample }
}

fn main() -> Result<(), slint::PlatformError> {
    RatingControlExample::new()?.run()
}
