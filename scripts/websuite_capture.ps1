#Requires -Version 5.1
<#
.SYNOPSIS
    Capture frames for all websuite cases

.DESCRIPTION
    This script runs hiwave-smoke for each case in the websuite manifest
    and captures frames for visual regression testing.

.PARAMETER OutputDir
    Directory to store captures (default: websuite\captures)

.EXAMPLE
    .\scripts\websuite_capture.ps1
    .\scripts\websuite_capture.ps1 -OutputDir "C:\captures"
#>

param(
    [string]$OutputDir
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$Manifest = Join-Path $ProjectDir "websuite\manifest.json"
if (-not $OutputDir) {
    $OutputDir = Join-Path $ProjectDir "websuite\captures"
}
$SmokeBin = Join-Path $ProjectDir "target\release\hiwave-smoke.exe"

# Check manifest exists
if (-not (Test-Path $Manifest)) {
    Write-Host "Error: Manifest not found at $Manifest" -ForegroundColor Red
    exit 1
}

# Build hiwave-smoke if needed
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

Write-Host "WebSuite Capture - Starting..."
Write-Host "Output directory: $OutputDir"
Write-Host ""

# Read manifest
$ManifestData = Get-Content $Manifest | ConvertFrom-Json
$DefaultViewport = $ManifestData.viewport

$Total = 0
$Passed = 0
$Failed = 0

foreach ($case in $ManifestData.cases) {
    $Total++

    $CaseId = $case.id
    $CasePath = $case.path

    # Use case viewport or fall back to default
    $Viewport = if ($case.viewport) { $case.viewport } else { $DefaultViewport }
    $Width = if ($Viewport -and $Viewport.width) { $Viewport.width } else { 800 }
    $Height = if ($Viewport -and $Viewport.height) { $Viewport.height } else { 600 }

    $HtmlFile = Join-Path $ProjectDir "websuite\$CasePath"
    $OutputFile = Join-Path $OutputDir "$CaseId.ppm"
    $PerfFile = Join-Path $OutputDir "$CaseId.perf.json"

    Write-Host "[$Total] Capturing: $CaseId ($Width x $Height)"

    if (-not (Test-Path $HtmlFile)) {
        Write-Host "  SKIP: HTML file not found: $HtmlFile" -ForegroundColor Yellow
        $Failed++
        continue
    }

    try {
        $output = & $SmokeBin `
            --html-file $HtmlFile `
            --width $Width `
            --height $Height `
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
Write-Host "WebSuite Capture Complete"
Write-Host "========================="
Write-Host "Total:  $Total"
Write-Host "Passed: $Passed"
Write-Host "Failed: $Failed"

# Generate summary JSON using Python
$OutputDirEscaped = $OutputDir -replace '\\', '\\\\'
$pythonScript = @"
import json
import os
from datetime import datetime

output_dir = r'$OutputDir'

summary = {
    'timestamp': datetime.now().isoformat(),
    'total': $Total,
    'passed': $Passed,
    'failed': $Failed,
    'captures': []
}

for f in os.listdir(output_dir):
    if f.endswith('.ppm'):
        case_id = f.replace('.ppm', '')
        perf_file = os.path.join(output_dir, case_id + '.perf.json')
        perf = {}
        if os.path.exists(perf_file):
            with open(perf_file) as pf:
                perf = json.load(pf)
        summary['captures'].append({
            'case_id': case_id,
            'frame': f,
            'perf': perf.get('perf', {})
        })

with open(os.path.join(output_dir, 'summary.json'), 'w') as f:
    json.dump(summary, f, indent=2)

print(f'Summary written to {output_dir}\\summary.json')
"@

& python -c $pythonScript

if ($Failed -gt 0) {
    exit 1
}
