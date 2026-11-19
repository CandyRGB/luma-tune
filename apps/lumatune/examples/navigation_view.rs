#[path = "../src/navigation_view.rs"]
mod navigation_view;

slint::slint! {
    import {
        NavigationViewBehavior, NavigationViewDisplayMode, NavigationViewExample, NavigationViewItem,
        NavigationViewItemKind, NavigationViewMeasurement, NavigationViewPaneDisplayMode,
        NavigationViewRow, NavigationViewTopLayout,
    } from "../../examples/winui3/navigation_view.slint";
    export {
        NavigationViewBehavior, NavigationViewDisplayMode, NavigationViewExample, NavigationViewItem,
        NavigationViewItemKind, NavigationViewMeasurement, NavigationViewPaneDisplayMode,
        NavigationViewRow, NavigationViewTopLayout,
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let window = NavigationViewExample::new()?;
    navigation_view::install(window.global::<NavigationViewBehavior>());
    if std::env::args().any(|argument| argument == "--top") {
        window.set_mode(NavigationViewPaneDisplayMode::Top);
    }
    window.run()
}
