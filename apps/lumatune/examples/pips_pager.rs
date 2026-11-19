slint::slint! {
    import { PipsPagerExample } from "../../examples/winui3/pips_pager.slint";
    export { PipsPagerExample }
}

fn main() -> Result<(), slint::PlatformError> {
    PipsPagerExample::new()?.run()
}
