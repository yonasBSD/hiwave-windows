//! Build script for webkit-wincairo-sys
//!
//! Links to WebKit DLLs installed at WEBKIT_PATH (default: C:\WebKit)

fn main() {
    // Only build on Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        return;
    }

    // Get WebKit installation path from environment or use default
    let webkit_path = std::env::var("WEBKIT_PATH").unwrap_or_else(|_| "C:\\WebKit".to_string());

    // Add library search paths
    println!("cargo:rustc-link-search=native={}/bin64", webkit_path);
    println!("cargo:rustc-link-search=native={}/lib64", webkit_path);

    // Link to WebKit libraries
    // Note: These are the primary DLLs needed for WinCairo WebKit
    println!("cargo:rustc-link-lib=dylib=WebKit");
    println!("cargo:rustc-link-lib=dylib=JavaScriptCore");

    // Re-run build if these change
    println!("cargo:rerun-if-env-changed=WEBKIT_PATH");
    println!("cargo:rerun-if-changed=build.rs");

    // Export include path for downstream crates
    println!("cargo:include={}/include", webkit_path);
}
