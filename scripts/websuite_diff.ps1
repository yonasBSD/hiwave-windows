#Requires -Version 5.1
<#
.SYNOPSIS
    Full websuite visual diff workflow

.DESCRIPTION
    This script:
    1. Captures RustKit frames
    2. Captures Chromium baselines (if needed)
    3. Compares and generates diff report

.PARAMETER RegenerateBaseline
    Force regeneration of Chromium baselines

.EXAMPLE
    .\scripts\websuite_diff.ps1
    .\scripts\websuite_diff.ps1 -RegenerateBaseline
#>

param(
    [switch]$RegenerateBaseline
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$BaselineTool = Join-Path $ProjectDir "tools\websuite-baseline"

Write-Host "WebSuite Visual Diff Workflow"
Write-Host "=============================="
Write-Host ""

# Step 1: Capture RustKit frames
Write-Host "Step 1: Capture RustKit frames"
Write-Host "------------------------------"
& "$ScriptDir\websuite_capture.ps1"
if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne $null) {
    Write-Warning "Some captures failed, continuing..."
}
Write-Host ""

# Step 2: Ensure baselines exist
$BaselineDir = Join-Path $ProjectDir "websuite\baselines"
if (-not (Test-Path $BaselineDir) -or $RegenerateBaseline) {
    Write-Host "Step 2: Generate Chromium baselines"
    Write-Host "------------------------------------"

    Push-Location $BaselineTool
    try {
        # Install dependencies if needed
        if (-not (Test-Path "node_modules")) {
            Write-Host "Installing dependencies..."
            & npm install
            & npx playwright install chromium
        }

        # Capture baselines
        & node capture.js
    } finally {
        Pop-Location
    }
    Write-Host ""
} else {
    Write-Host "Step 2: Using existing baselines (use -RegenerateBaseline to refresh)"
    Write-Host ""
}

# Step 3: Compare
Write-Host "Step 3: Compare RustKit vs Chromium"
Write-Host "------------------------------------"
Push-Location $BaselineTool
try {
    # Install pngjs if needed
    try {
        & node -e "require('pngjs')" 2>$null
    } catch {
        & npm install pngjs
    }

    & node compare.js
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "Done! Check websuite\diffs\ for results."
