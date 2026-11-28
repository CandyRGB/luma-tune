fn main() {
    #[cfg(all(windows, target_env = "msvc"))]
    {
        // The build dependency is only present when targeting MSVC.
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
            return;
        }
        println!("cargo:rerun-if-changed=src/postprocess.cpp");
        cc::Build::new()
            .cpp(true)
            .std("c++20")
            .flag("/EHsc")
            .flag("/utf-8")
            .file("src/postprocess.cpp")
            .compile("lumatune_postprocess");
        for library in [
            "d3d11",
            "dxgi",
            "d3dcompiler",
            "dwmapi",
            "runtimeobject",
            "ole32",
            "user32",
        ] {
            println!("cargo:rustc-link-lib={library}");
        }
    }
}
