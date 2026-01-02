//! Build script for webkit-wincairo-sys
//!
//! Links to WebKit DLLs and copies them to the output directory.
//!
//! DLL search order:
//! 1. Bundled DLLs in deps/wincairo/ (relative to workspace root)
//! 2. WEBKIT_PATH environment variable (default: C:\WebKit)

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Core WebKit DLLs that must be present
/// Note: WinCairo uses WebKit2.dll (not WebKit.dll)
const CORE_DLLS: &[&str] = &["WebKit2.dll", "JavaScriptCore.dll"];

/// Additional DLLs that may be needed at runtime
#[allow(dead_code)]
const OPTIONAL_DLLS: &[&str] = &[
    "WebCore.dll",
    "WTF.dll",
    "WebKitLegacy.dll",
    // ANGLE (OpenGL ES) libraries
    "libGLESv2.dll",
    "libEGL.dll",
    // WebKitRequirements dependencies
    "cairo.dll",
    "libcurl.dll",
    "libpng16.dll",
    "zlib1.dll",
    "libjpeg-9.dll",
    "libxml2.dll",
    "libxslt.dll",
    "sqlite3.dll",
    "libwebp.dll",
    "pthreadVC3.dll",
    "brotlicommon.dll",
    "brotlidec.dll",
    "nghttp2.dll",
    "libssh2.dll",
];

fn main() {
    // Only build on Windows
    if env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        return;
    }

    // Re-run triggers
    println!("cargo:rerun-if-env-changed=WEBKIT_PATH");
    println!("cargo:rerun-if-changed=build.rs");

    // Find WebKit installation
    let webkit_path = find_webkit_path();

    match webkit_path {
        Some(path) => {
            println!("cargo:warning=Using WebKit from: {}", path.display());

            // Add library search paths
            let bin_path = path.join("bin64");
            let lib_path = path.join("lib64");

            if bin_path.exists() {
                println!("cargo:rustc-link-search=native={}", bin_path.display());
            } else {
                // DLLs might be in root (bundled case)
                println!("cargo:rustc-link-search=native={}", path.display());
            }

            if lib_path.exists() {
                println!("cargo:rustc-link-search=native={}", lib_path.display());
            } else {
                println!("cargo:rustc-link-search=native={}", path.display());
            }

            // Generate import libraries if they don't exist
            // WebKit buildbot archives often don't include .lib files
            generate_import_libs(&path);

            // Link to WebKit libraries
            // Note: WinCairo uses WebKit2, not WebKit
            println!("cargo:rustc-link-lib=dylib=WebKit2");
            println!("cargo:rustc-link-lib=dylib=JavaScriptCore");

            // Export include path for downstream crates
            let include_path = path.join("include");
            if include_path.exists() {
                println!("cargo:include={}", include_path.display());
            }

            // Copy DLLs to output directory
            copy_dlls_to_output(&path);
        }
        None => {
            println!("cargo:warning=WebKit not found. Build will fail at link time.");
            println!("cargo:warning=Run 'scripts/setup-wincairo.ps1' to install WebKit DLLs,");
            println!("cargo:warning=or set WEBKIT_PATH environment variable.");

            // Still emit link directives so we get a clearer error
            println!("cargo:rustc-link-lib=dylib=WebKit2");
            println!("cargo:rustc-link-lib=dylib=JavaScriptCore");
        }
    }
}

/// Find WebKit installation path
///
/// Search order:
/// 1. Bundled DLLs in deps/wincairo/
/// 2. WEBKIT_PATH environment variable
/// 3. Default path C:\WebKit
fn find_webkit_path() -> Option<PathBuf> {
    // Try bundled DLLs first
    if let Some(bundled) = find_bundled_webkit() {
        return Some(bundled);
    }

    // Try WEBKIT_PATH environment variable
    if let Ok(webkit_path) = env::var("WEBKIT_PATH") {
        let path = PathBuf::from(&webkit_path);
        if has_webkit_dlls(&path) {
            return Some(path);
        }
    }

    // Try default path
    let default_path = PathBuf::from("C:\\WebKit");
    if has_webkit_dlls(&default_path) {
        return Some(default_path);
    }

    None
}

/// Find bundled WebKit DLLs in the workspace
fn find_bundled_webkit() -> Option<PathBuf> {
    // Get the manifest directory (where Cargo.toml is)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    let manifest_path = PathBuf::from(manifest_dir);

    // Go up to workspace root (crates/webkit-wincairo-sys -> workspace)
    let workspace_root = manifest_path.parent()?.parent()?;

    // Check deps/wincairo/
    let bundled_path = workspace_root.join("deps").join("wincairo");

    // Re-run if bundled DLLs change
    if bundled_path.exists() {
        println!("cargo:rerun-if-changed={}", bundled_path.display());
    }

    if has_webkit_dlls(&bundled_path) {
        return Some(bundled_path);
    }

    None
}

/// Check if a directory contains the core WebKit DLLs
fn has_webkit_dlls(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    // Check in the root directory
    for dll in CORE_DLLS {
        if path.join(dll).exists() {
            return true;
        }
    }

    // Check in bin64 subdirectory
    let bin64 = path.join("bin64");
    if bin64.exists() {
        for dll in CORE_DLLS {
            if bin64.join(dll).exists() {
                return true;
            }
        }
    }

    false
}

/// Copy WebKit DLLs to the cargo output directory
fn copy_dlls_to_output(webkit_path: &Path) {
    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => return,
    };

    // The OUT_DIR is something like target/debug/build/webkit-wincairo-sys-xxx/out
    // We need to copy to target/debug/ (or target/release/)
    // Go up to find the target profile directory
    let target_dir = out_dir.ancestors().find(|p| {
        p.file_name()
            .map(|n| n == "debug" || n == "release")
            .unwrap_or(false)
    });

    let target_dir = match target_dir {
        Some(dir) => dir.to_path_buf(),
        None => return,
    };

    // Find DLLs/EXEs in webkit_path or subdirectories
    // WebKit helper processes (WebKitWebProcess.exe etc) may be in webkit-extracted/bin
    let dll_sources = [
        webkit_path.to_path_buf(),
        webkit_path.join("bin64"),
        webkit_path.join("webkit-extracted").join("bin"),
    ];

    // Copy ALL DLLs from source directories
    for source_dir in &dll_sources {
        if !source_dir.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(source_dir) {
            for entry in entries.filter_map(Result::ok) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Copy all DLL and EXE files (WebKit helper processes are required)
                if name_str.ends_with(".dll") || name_str.ends_with(".exe") {
                    let src = entry.path();
                    let dst = target_dir.join(&name);

                    // Only copy if source is newer or destination doesn't exist
                    let should_copy = if dst.exists() {
                        match (src.metadata(), dst.metadata()) {
                            (Ok(src_meta), Ok(dst_meta)) => {
                                src_meta.modified().ok() > dst_meta.modified().ok()
                            }
                            _ => true,
                        }
                    } else {
                        true
                    };

                    if should_copy {
                        if let Err(e) = fs::copy(&src, &dst) {
                            println!("cargo:warning=Failed to copy {}: {}", name_str, e);
                        } else {
                            println!(
                                "cargo:warning=Copied {} to {}",
                                name_str,
                                target_dir.display()
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Generate import libraries (.lib) from DLLs if they don't exist
///
/// WebKit buildbot archives often don't include import libraries,
/// so we generate them using MSVC tools (dumpbin + lib).
fn generate_import_libs(webkit_path: &Path) {
    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => return,
    };

    // Libraries we need to generate
    let libs = [
        ("WebKit2.dll", "WebKit2"),
        ("JavaScriptCore.dll", "JavaScriptCore"),
    ];

    // Find DLLs in webkit_path or subfolders
    let search_dirs = [
        webkit_path.to_path_buf(),
        webkit_path.join("bin64"),
        webkit_path.join("bin"),
    ];

    for (dll_name, lib_name) in &libs {
        let lib_file = out_dir.join(format!("{}.lib", lib_name));

        // Skip if lib already exists
        if lib_file.exists() {
            println!("cargo:rustc-link-search=native={}", out_dir.display());
            continue;
        }

        // Find the DLL
        let mut dll_path = None;
        for dir in &search_dirs {
            let candidate = dir.join(dll_name);
            if candidate.exists() {
                dll_path = Some(candidate);
                break;
            }
        }

        let dll_path = match dll_path {
            Some(p) => p,
            None => {
                println!(
                    "cargo:warning=Could not find {} to generate import library",
                    dll_name
                );
                continue;
            }
        };

        // Generate .def file using dumpbin
        let def_file = out_dir.join(format!("{}.def", lib_name));
        if let Err(e) = generate_def_file(&dll_path, &def_file, lib_name) {
            println!("cargo:warning=Failed to generate {}.def: {}", lib_name, e);
            continue;
        }

        // Generate .lib file using lib.exe
        if let Err(e) = generate_lib_file(&def_file, &lib_file) {
            println!("cargo:warning=Failed to generate {}.lib: {}", lib_name, e);
            continue;
        }

        println!("cargo:warning=Generated {}.lib from {}", lib_name, dll_name);
        println!("cargo:rustc-link-search=native={}", out_dir.display());
    }
}

/// Find MSVC tool (dumpbin or lib)
fn find_msvc_tool(tool_name: &str) -> Option<PathBuf> {
    // First try the tool directly (if in PATH)
    if Command::new(tool_name).arg("/?").output().is_ok() {
        return Some(PathBuf::from(tool_name));
    }

    // Try to find via cc crate's MSVC detection
    if let Some(tool) = cc::windows_registry::find_tool("x86_64-pc-windows-msvc", tool_name) {
        return Some(tool.path().to_path_buf());
    }

    // Try common Visual Studio paths
    let vs_paths = [
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC",
        r"C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC",
        r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Tools\MSVC",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC",
        r"P:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC",
    ];

    for vs_base in &vs_paths {
        let base = Path::new(vs_base);
        if !base.exists() {
            continue;
        }

        // Find the latest MSVC version
        if let Ok(entries) = fs::read_dir(base) {
            let mut versions: Vec<_> = entries
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .collect();
            versions.sort_by_key(|b| std::cmp::Reverse(b.file_name()));

            for version_dir in versions {
                let tool_path = version_dir
                    .path()
                    .join("bin")
                    .join("Hostx64")
                    .join("x64")
                    .join(format!("{}.exe", tool_name));

                if tool_path.exists() {
                    return Some(tool_path);
                }
            }
        }
    }

    None
}

/// Generate a .def file from a DLL using dumpbin
fn generate_def_file(dll_path: &Path, def_path: &Path, lib_name: &str) -> Result<(), String> {
    let dumpbin = find_msvc_tool("dumpbin")
        .ok_or_else(|| "dumpbin not found - install Visual Studio with C++ tools".to_string())?;

    // Run dumpbin /exports
    let output = Command::new(&dumpbin)
        .args(["/exports", dll_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to run dumpbin: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "dumpbin failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse dumpbin output and extract exports
    // Format: "ordinal hint RVA      name"
    // Example: "          1    0 00026600 ?GPUProcessMain@WebKit@@YAHHPEAPEAD@Z"
    let mut exports = Vec::new();
    let mut in_exports = false;

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Look for the header line that marks start of exports
        if trimmed.starts_with("ordinal") && trimmed.contains("hint") && trimmed.contains("name") {
            in_exports = true;
            continue;
        }

        if !in_exports {
            continue;
        }

        // Empty line after exports section
        if trimmed.is_empty() {
            // Could be blank line between header and data, continue
            continue;
        }

        // Summary section ends the exports
        if trimmed.contains("Summary") {
            break;
        }

        // Parse export line: "ordinal hint RVA name"
        // The line has 4+ columns: ordinal, hint, RVA, name (name might have spaces in decorated form)
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 4 {
            // First part should be a number (ordinal)
            if parts[0].parse::<u32>().is_ok() {
                // The 4th part onwards is the function name
                let name = parts[3];
                // Skip forwarded exports (contain "=")
                if !name.contains('=') && !name.starts_with('[') {
                    exports.push(name.to_string());
                }
            }
        }
    }

    if exports.is_empty() {
        return Err("No exports found in DLL".to_string());
    }

    // Write .def file
    let mut def_content = format!("LIBRARY {}\nEXPORTS\n", lib_name);
    for export in &exports {
        def_content.push_str(&format!("    {}\n", export));
    }

    fs::write(def_path, def_content).map_err(|e| format!("Failed to write .def file: {}", e))?;

    Ok(())
}

/// Generate a .lib file from a .def file using lib.exe
fn generate_lib_file(def_path: &Path, lib_path: &Path) -> Result<(), String> {
    let lib_exe = find_msvc_tool("lib")
        .ok_or_else(|| "lib.exe not found - install Visual Studio with C++ tools".to_string())?;

    let output = Command::new(&lib_exe)
        .args([
            &format!("/def:{}", def_path.display()),
            &format!("/out:{}", lib_path.display()),
            "/machine:x64",
        ])
        .output()
        .map_err(|e| format!("Failed to run lib.exe: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "lib.exe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}
