#Requires -Version 5.1
<#
.SYNOPSIS
    Capture RustKit renders for static web suite

.DESCRIPTION
    This script runs each static web suite test case through RustKit
    and captures the rendered output for validation.

.PARAMETER OutputDir
    Directory to store captures (default: static-suite-captures)

.EXAMPLE
    .\scripts\static_suite_capture.ps1
    .\scripts\static_suite_capture.ps1 -OutputDir "C:\captures"
#>

param(
    [string]$OutputDir
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$SuiteDir = Join-Path $ProjectDir "static-web-suite"
$Manifest = Join-Path $SuiteDir "manifest.json"
if (-not $OutputDir) {
    $OutputDir = Join-Path $ProjectDir "static-suite-captures"
}
$SmokeBin = Join-Path $ProjectDir "target\release\hiwave-smoke.exe"

Write-Host "Static Web Suite Capture"
Write-Host "========================"
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

# Check manifest exists
if (-not (Test-Path $Manifest)) {
    Write-Host "Error: Manifest not found at $Manifest" -ForegroundColor Red
    exit 1
}

# Read manifest
$ManifestData = Get-Content $Manifest | ConvertFrom-Json

$Total = 0
$Passed = 0
$Failed = 0

foreach ($case in $ManifestData.cases) {
    $Total++

    $CaseId = $case.id
    $CasePath = $case.path
    $Width = $case.viewport.width
    $Height = $case.viewport.height

    $HtmlPath = Join-Path $SuiteDir $CasePath
    $OutputFile = Join-Path $OutputDir "$CaseId.ppm"
    $PerfFile = Join-Path $OutputDir "$CaseId.perf.json"

    Write-Host "[$Total] Capturing: $CaseId ($Width x $Height)"

    if (-not (Test-Path $HtmlPath)) {
        Write-Host "  SKIP: File not found: $HtmlPath" -ForegroundColor Yellow
        $Failed++
        continue
    }

    try {
        $output = & $SmokeBin `
            --html-file $HtmlPath `
            --width $Width `
            --height $Height `
            --duration-ms 1000 `
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
Write-Host "Static Web Suite Capture Complete"
Write-Host "=================================="
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
suite_dir = r'$SuiteDir'
manifest_file = r'$Manifest'

# Load manifest
with open(manifest_file) as f:
    manifest = json.load(f)

try:
    git_sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], stderr=subprocess.DEVNULL).decode().strip()
except:
    git_sha = 'unknown'

summary = {
    "timestamp": datetime.now().isoformat(),
    "git_sha": git_sha,
    "renderer": "rustkit",
    "suite": "static-web-suite",
    "total": len(manifest["cases"]),
    "passed": $Passed,
    "failed": $Failed,
    "captures": []
}

for case in manifest["cases"]:
    ppm_file = os.path.join(output_dir, f"{case['id']}.ppm")
    perf_file = os.path.join(output_dir, f"{case['id']}.perf.json")

    perf = {}
    if os.path.exists(perf_file):
        with open(perf_file) as f:
            data = json.load(f)
            perf = data.get("perf") or data.get("timings", {})

    summary["captures"].append({
        "case_id": case["id"],
        "name": case["name"],
        "source_file": case["path"],
        "viewport": case["viewport"],
        "frame": f"{case['id']}.ppm" if os.path.exists(ppm_file) else None,
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
