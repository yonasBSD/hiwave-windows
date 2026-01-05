# run-rustkit.ps1 - Run HiWave with RustKit hybrid mode
#
# This script builds and runs HiWave using RustKit for content rendering
# with WRY (WebView2) for Chrome UI and Shelf components.
#
# Features:
# - RustKit: Pure Rust browser engine for content rendering
# - WRY: System WebView2 for Chrome UI (tabs, address bar)
# - Engine-level ad blocking via shield adapter
# - Hardware-accelerated GPU rendering via wgpu
#
# Usage:
#   .\scripts\run-rustkit.ps1 [cargo-args...]
#
# Examples:
#   .\scripts\run-rustkit.ps1           # Build and run (debug)
#   .\scripts\run-rustkit.ps1 --release # Build and run (release)

$ErrorActionPreference = "Stop"

Push-Location $PSScriptRoot\..

Write-Host "Building HiWave with RustKit hybrid mode..." -ForegroundColor Cyan
cargo build -p hiwave-app --no-default-features --features rustkit $args

Write-Host "Running HiWave (RustKit hybrid mode)..." -ForegroundColor Green
cargo run -p hiwave-app --no-default-features --features rustkit $args

Pop-Location

