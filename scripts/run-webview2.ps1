# run-webview2.ps1 - Run HiWave with WebView2 fallback (no RustKit)
#
# This script builds and runs HiWave using Microsoft Edge WebView2 for
# all rendering, bypassing the RustKit engine entirely.
#
# Features:
# - WebView2: Microsoft Edge Chromium-based WebView for all rendering
# - WRY/Tao: Cross-platform window and WebView management
# - Full compatibility with Windows system features
#
# Use this mode when:
# - Debugging issues that might be RustKit-specific
# - Testing compatibility with WebView2
# - Comparing rendering behavior between engines
#
# Usage:
#   .\scripts\run-webview2.ps1 [cargo-args...]
#
# Examples:
#   .\scripts\run-webview2.ps1           # Build and run (debug)
#   .\scripts\run-webview2.ps1 --release # Build and run (release)

$ErrorActionPreference = "Stop"

Push-Location $PSScriptRoot\..

Write-Host "Building HiWave with WebView2 fallback..." -ForegroundColor Cyan
cargo build -p hiwave-app --no-default-features --features webview-fallback $args

Write-Host "Running HiWave (WebView2 fallback mode)..." -ForegroundColor Green
cargo run -p hiwave-app --no-default-features --features webview-fallback $args

Pop-Location

