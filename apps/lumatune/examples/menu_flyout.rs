#[path = "../src/menu_flyout.rs"]
mod menu_flyout;

slint::slint! {
    import { MenuFlyoutExample, MenuFlyoutBehavior, MenuFlyoutItem, MenuFlyoutItemKind }
        from "../../examples/winui3/menu_flyout.slint";
    export { MenuFlyoutExample, MenuFlyoutBehavior, MenuFlyoutItem, MenuFlyoutItemKind }
}

fn main() -> Result<(), slint::PlatformError> {
    let window = MenuFlyoutExample::new()?;
    menu_flyout::install(window.global::<MenuFlyoutBehavior>());
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    window.set_dark(arguments.iter().any(|arg| arg == "--dark"));
    window.set_compact(arguments.iter().any(|arg| arg == "--compact"));
    window.run()
}
