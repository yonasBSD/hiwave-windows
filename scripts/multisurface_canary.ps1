# multisurface_canary.ps1 - Test multisurface RustKit rendering
#
# This script captures frames from both chrome and content RustKit views
# to verify multisurface compositing works correctly.
#
# Usage: .\scripts\multisurface_canary.ps1 [-OutputDir <path>]

param(
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
if (-not $OutputDir) {
    $OutputDir = Join-Path $ProjectRoot "multisurface-captures"
}
$SmokeBin = Join-Path $ProjectRoot "target\release\hiwave-smoke.exe"

Write-Host "Multisurface Canary Test" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan
Write-Host ""

# Build if needed
if (-not (Test-Path $SmokeBin)) {
    Write-Host "Building hiwave-smoke..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    cargo build -p hiwave-smoke --release
    Pop-Location
}

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Capture chrome view
Write-Host "[1/2] Capturing Chrome view..." -ForegroundColor White
$chromeFixture = Join-Path $ProjectRoot "fixtures\multisurface_chrome.html"
$chromePpm = Join-Path $OutputDir "chrome.ppm"
$chromePerf = Join-Path $OutputDir "chrome.perf.json"

$chromeOk = $false
if (Test-Path $chromeFixture) {
    $process = Start-Process -FilePath $SmokeBin -ArgumentList @(
        "--html-file", $chromeFixture,
        "--width", 1100,
        "--height", 72,
        "--duration-ms", 500,
        "--dump-frame", $chromePpm,
        "--perf-output", $chromePerf
    ) -NoNewWindow -Wait -PassThru 2>$null
    
    if (Test-Path $chromePpm) {
        Write-Host "  OK: Chrome frame captured" -ForegroundColor Green
        $chromeOk = $true
    } else {
        Write-Host "  FAIL: Chrome frame not generated" -ForegroundColor Red
    }
} else {
    Write-Host "  SKIP: multisurface_chrome.html not found" -ForegroundColor Yellow
}

# Capture content view
Write-Host "[2/2] Capturing Content view..." -ForegroundColor White
$contentFixture = Join-Path $ProjectRoot "fixtures\multisurface_content.html"
$contentPpm = Join-Path $OutputDir "content.ppm"
$contentPerf = Join-Path $OutputRoot "content.perf.json"

$contentOk = $false
if (Test-Path $contentFixture) {
    $process = Start-Process -FilePath $SmokeBin -ArgumentList @(
        "--html-file", $contentFixture,
        "--width", 1100,
        "--height", 600,
        "--duration-ms", 500,
        "--dump-frame", $contentPpm,
        "--perf-output", $contentPerf
    ) -NoNewWindow -Wait -PassThru 2>$null
    
    if (Test-Path $contentPpm) {
        Write-Host "  OK: Content frame captured" -ForegroundColor Green
        $contentOk = $true
    } else {
        Write-Host "  FAIL: Content frame not generated" -ForegroundColor Red
    }
} else {
    Write-Host "  SKIP: multisurface_content.html not found" -ForegroundColor Yellow
}

# Generate summary
Write-Host ""
Write-Host "Multisurface Canary Complete" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan

# Load perf data if available
$chromePerf = @{}
$contentPerf = @{}

$chromePerfFile = Join-Path $OutputDir "chrome.perf.json"
$contentPerfFile = Join-Path $OutputDir "content.perf.json"

if (Test-Path $chromePerfFile) {
    $chromePerf = (Get-Content $chromePerfFile | ConvertFrom-Json).perf
}
if (Test-Path $contentPerfFile) {
    $contentPerf = (Get-Content $contentPerfFile | ConvertFrom-Json).perf
}

$summary = @{
    timestamp = (Get-Date).ToUniversalTime().ToString("o")
    status = if ($chromeOk -and $contentOk) { "pass" } else { "fail" }
    surfaces = @{
        chrome = @{
            status = if ($chromeOk) { "ok" } else { "fail" }
            dimensions = @{ width = 1100; height = 72 }
            frame = if ($chromeOk) { "chrome.ppm" } else { $null }
            perf = $chromePerf
        }
        content = @{
            status = if ($contentOk) { "ok" } else { "fail" }
            dimensions = @{ width = 1100; height = 600 }
            frame = if ($contentOk) { "content.ppm" } else { $null }
            perf = $contentPerf
        }
    }
    notes = "Separate captures - compositor integration pending"
} | ConvertTo-Json -Depth 10

$summaryPath = Join-Path $OutputDir "summary.json"
$summary | Out-File -FilePath $summaryPath -Encoding utf8

Write-Host "Chrome:  $(if ($chromeOk) { 'PASS' } else { 'FAIL' })" -ForegroundColor $(if ($chromeOk) { 'Green' } else { 'Red' })
Write-Host "Content: $(if ($contentOk) { 'PASS' } else { 'FAIL' })" -ForegroundColor $(if ($contentOk) { 'Green' } else { 'Red' })
Write-Host "Overall: $($summary | ConvertFrom-Json | Select-Object -ExpandProperty status)" -ForegroundColor $(if ($chromeOk -and $contentOk) { 'Green' } else { 'Red' })
Write-Host ""
Write-Host "Results: $OutputDir"

# Exit with appropriate code
if ($chromeOk -and $contentOk) {
    exit 0
} else {
    exit 1
}

