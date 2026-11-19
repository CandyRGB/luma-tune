fn main() {
    slint_build::compile("ui/app.slint").expect("slint build failed");
    // NavigationView uses the named slots supported by the pinned Slint compiler.
    println!("cargo:rustc-env=SLINT_ENABLE_EXPERIMENTAL_FEATURES=1");
}
