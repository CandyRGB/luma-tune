use std::{collections::HashMap, env};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=lumatune.rc");
        println!("cargo:rerun-if-changed=ui/icons/lumatune.ico");
        let version = env!("CARGO_PKG_VERSION");
        let file_version = format!(
            "{},{},{},0",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
        );
        let macros = [
            format!("LUMATUNE_VERSION={version}"),
            format!("LUMATUNE_FILE_VERSION={file_version}"),
        ];
        embed_resource::compile_for("lumatune.rc", ["lumatune"], macros)
            .manifest_required()
            .expect("could not embed the Windows application icon");
    }

    // The WinUI components use Slint's experimental named slots.
    // SAFETY: this build script sets the environment before starting any threads.
    unsafe { env::set_var("SLINT_ENABLE_EXPERIMENTAL_FEATURES", "1") };
    let winui = slint_winui::library_path();
    let libraries = HashMap::from([
        ("winui-icons".into(), winui.with_file_name("icons")),
        ("winui".into(), winui),
    ]);
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(libraries)
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
    slint_build::compile_with_config("ui/app.slint", config).expect("slint build failed");
    slint_winui::write_behavior_modules(
        env::var_os("OUT_DIR").expect("Cargo did not set OUT_DIR"),
        &[
            "breadcrumb_bar",
            "combo_box",
            "navigation_view",
            "number_box",
            "radio_buttons",
            "window_title_bar",
        ],
    )
    .expect("could not register Slint WinUI behavior modules");
}
