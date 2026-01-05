# run-native-win32.ps1 - Run HiWave with 100% RustKit (default)
#
# This script builds and runs HiWave using RustKit for ALL rendering:
# - Chrome UI (tabs, address bar, sidebar)
# - Content WebView (web pages)
# - Shelf (command palette)
#
# No WRY, no Tao, no WebView2 - pure RustKit rendering via Win32 API.
#
# Features:
# - 100% Rust rendering pipeline
# - Engine-level ad blocking via shield adapter
# - Hardware-accelerated GPU rendering via wgpu
# - Direct Win32 window management
#
# Usage:
#   .\scripts\run-native-win32.ps1 [cargo-args...]
#
# Examples:
#   .\scripts\run-native-win32.ps1           # Build and run (debug)
#   .\scripts\run-native-win32.ps1 --release # Build and run (release)

$ErrorActionPreference = "Stop"

Push-Location $PSScriptRoot\..

Write-Host "Building HiWave with native-win32 (100% RustKit)..." -ForegroundColor Cyan
cargo build -p hiwave-app --features native-win32 $args

Write-Host "Running HiWave (native-win32 mode)..." -ForegroundColor Green
cargo run -p hiwave-app --features native-win32 $args

Pop-Location

