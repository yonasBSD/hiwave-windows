#Requires -Version 5.1
<#
.SYNOPSIS
    Capture built-in pages with RustKit engine

.DESCRIPTION
    This script captures deterministic frames for all built-in HiWave pages
    using the RustKit engine at standardized viewports.

.PARAMETER OutputDir
    Directory to store captures (default: builtins-captures)

.EXAMPLE
    .\scripts\builtins_capture.ps1
    .\scripts\builtins_capture.ps1 -OutputDir "C:\captures"
#>

param(
    [string]$OutputDir
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$UiDir = Join-Path $ProjectDir "crates\hiwave-app\src\ui"
if (-not $OutputDir) {
    $OutputDir = Join-Path $ProjectDir "builtins-captures"
}
$SmokeBin = Join-Path $ProjectDir "target\release\hiwave-smoke.exe"

Write-Host "Built-in Pages Capture (RustKit)"
Write-Host "================================="
Write-Host ""

# Build if needed
if (-not (Test-Path $SmokeBin)) {
    Write-Host "Building hiwave-smoke..."
    Push-Location $ProjectDir
    try {
        & cargo build -p hiwave-smoke --release
        if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    } finally {
        Pop-Location
    }
}

# Create output directory
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

# Define built-in pages to capture
$Pages = @(
    @{ Id = "new_tab"; File = "new_tab.html"; Width = 1280; Height = 800 },
    @{ Id = "about"; File = "about.html"; Width = 1280; Height = 800 },
    @{ Id = "settings"; File = "settings.html"; Width = 1280; Height = 800 },
    @{ Id = "chrome"; File = "chrome.html"; Width = 1280; Height = 72 },
    @{ Id = "shelf"; File = "shelf.html"; Width = 1280; Height = 120 }
)

$Total = 0
$Passed = 0
$Failed = 0

foreach ($page in $Pages) {
    $Total++

    $HtmlPath = Join-Path $UiDir $page.File
    $OutputFile = Join-Path $OutputDir "$($page.Id).ppm"
    $PerfFile = Join-Path $OutputDir "$($page.Id).perf.json"

    Write-Host "[$Total/$($Pages.Count)] Capturing: $($page.Id) ($($page.Width) x $($page.Height))"

    if (-not (Test-Path $HtmlPath)) {
        Write-Host "  SKIP: File not found: $HtmlPath" -ForegroundColor Yellow
        $Failed++
        continue
    }

    try {
        $output = & $SmokeBin `
            --html-file $HtmlPath `
            --width $page.Width `
            --height $page.Height `
            --duration-ms 500 `
            --dump-frame $OutputFile `
            --perf-output $PerfFile 2>&1

        if ($LASTEXITCODE -eq 0 -and (Test-Path $OutputFile)) {
            Write-Host "  OK: Frame captured" -ForegroundColor Green
            $Passed++
        } else {
            Write-Host "  FAIL: Frame not generated" -ForegroundColor Red
            $Failed++
        }
    } catch {
        Write-Host "  FAIL: hiwave-smoke exited with error" -ForegroundColor Red
        $Failed++
    }
}

Write-Host ""
Write-Host "Built-in Pages Capture Complete"
Write-Host "================================"
Write-Host "Total:  $Total"
Write-Host "Passed: $Passed"
Write-Host "Failed: $Failed"

# Generate summary JSON using Python
$pythonScript = @"
import json
import os
import subprocess
from datetime import datetime

output_dir = r'$OutputDir'
pages = [
    {"id": "new_tab", "file": "new_tab.html", "viewport": {"width": 1280, "height": 800}},
    {"id": "about", "file": "about.html", "viewport": {"width": 1280, "height": 800}},
    {"id": "settings", "file": "settings.html", "viewport": {"width": 1280, "height": 800}},
    {"id": "chrome", "file": "chrome.html", "viewport": {"width": 1280, "height": 72}},
    {"id": "shelf", "file": "shelf.html", "viewport": {"width": 1280, "height": 120}},
]

try:
    git_sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], stderr=subprocess.DEVNULL).decode().strip()
except:
    git_sha = 'unknown'

summary = {
    "timestamp": datetime.now().isoformat(),
    "git_sha": git_sha,
    "renderer": "rustkit",
    "dpr": 2.0,
    "total": len(pages),
    "passed": $Passed,
    "failed": $Failed,
    "captures": []
}

for page in pages:
    ppm_file = os.path.join(output_dir, f"{page['id']}.ppm")
    perf_file = os.path.join(output_dir, f"{page['id']}.perf.json")

    perf = {}
    if os.path.exists(perf_file):
        with open(perf_file) as f:
            perf = json.load(f).get("perf", {})

    summary["captures"].append({
        "page_id": page["id"],
        "source_file": page["file"],
        "viewport": page["viewport"],
        "frame": f"{page['id']}.ppm" if os.path.exists(ppm_file) else None,
        "status": "ok" if os.path.exists(ppm_file) else "fail",
        "perf": perf
    })

with open(os.path.join(output_dir, "summary.json"), "w") as f:
    json.dump(summary, f, indent=2)

print(f"Summary written to {output_dir}\\summary.json")
"@

& python -c $pythonScript

if ($Failed -gt 0) {
    exit 1
}
