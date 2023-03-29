fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    if let Ok(rust_toolchain) = std::env::var("RUSTUP_TOOLCHAIN") {
        if rust_toolchain.starts_with("nightly") {
            println!(r#"cargo:rustc-cfg=feature="nightly-features""#);
        }
    }
}
