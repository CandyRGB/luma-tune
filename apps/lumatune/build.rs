fn main() {
    // Enable named slots for both the app compiler and the example macros.
    // SAFETY: this build script sets the environment before starting any threads.
    unsafe { std::env::set_var("SLINT_ENABLE_EXPERIMENTAL_FEATURES", "1") };
    slint_build::compile("ui/app.slint").expect("slint build failed");
    println!("cargo:rustc-env=SLINT_ENABLE_EXPERIMENTAL_FEATURES=1");
}
