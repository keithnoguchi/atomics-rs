//! Conditional compilation between stable and nightly
//!
//! https://stackoverflow.com/questions/59542378/conditional-compilation-for-nightly-vs-stable-rust-or-compiler-version

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    if let Ok(rust_toolchain) = env::var("RUSTUP_TOOLCHAIN") {
        if rust_toolchain.starts_with("nightly") {
            println!(r#"cargo:rustc-cfg=feature="nightly-features""#);
        }
    }
}
