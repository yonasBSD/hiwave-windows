#Requires -Version 5.1
<#
.SYNOPSIS
    Visual Test Runner - Shows each parity test case in a window

.DESCRIPTION
    Runs visual tests for built-in pages and websuite cases,
    displaying each in a window for visual inspection.

.PARAMETER Duration
    How long to show each page in milliseconds (default: 5000)

.PARAMETER Case
    Run only a single case by name

.EXAMPLE
    .\scripts\visual_test_runner.ps1
    .\scripts\visual_test_runner.ps1 -Duration 3000
    .\scripts\visual_test_runner.ps1 -Case new_tab
#>

param(
    [int]$Duration = 5000,
    [string]$Case,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ($Help) {
    Write-Host "Usage: .\visual_test_runner.ps1 [-Duration <ms>] [-Case <name>]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Duration <ms>   How long to show each page (default: 5000)"
    Write-Host "  -Case <name>     Run only a single case"
    Write-Host ""
    Write-Host "Available cases:"
    Write-Host "  Built-ins: new_tab, about, settings, chrome_rustkit, shelf"
    Write-Host "  Websuite: article-typography, card-grid, css-selectors,"
    Write-Host "            flex-positioning, form-elements, gradient-backgrounds,"
    Write-Host "            image-gallery, sticky-scroll"
    exit 0
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir

function Run-Case {
    param(
        [string]$Name,
        [string]$Html,
        [int]$Width,
        [int]$Height
    )

    Write-Host ""
    Write-Host "--- $Name ---"
    Write-Host "  File: $Html"
    Write-Host "  Size: ${Width}x${Height}"
    Write-Host "  Opening window for ${Duration}ms..."

    $HtmlPath = Join-Path $ProjectDir $Html

    try {
        & cargo run --release -p hiwave-smoke -- `
            --html-file $HtmlPath `
            --width $Width `
            --height $Height `
            --duration-ms $Duration 2>&1 | Out-Host

        if ($LASTEXITCODE -eq 0) {
            Write-Host "  Done" -ForegroundColor Green
            return $true
        } else {
            Write-Host "  Error occurred" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "  Error occurred: $_" -ForegroundColor Red
        return $false
    }
}

# Define all test cases
$TestCases = @{
    "new_tab" = @{ Html = "crates\hiwave-app\src\ui\new_tab.html"; Width = 1280; Height = 800 }
    "about" = @{ Html = "crates\hiwave-app\src\ui\about.html"; Width = 800; Height = 600 }
    "settings" = @{ Html = "crates\hiwave-app\src\ui\settings.html"; Width = 1024; Height = 768 }
    "chrome_rustkit" = @{ Html = "crates\hiwave-app\src\ui\chrome_rustkit.html"; Width = 1280; Height = 100 }
    "shelf" = @{ Html = "crates\hiwave-app\src\ui\shelf.html"; Width = 1280; Height = 120 }
    "article-typography" = @{ Html = "websuite\cases\article-typography\index.html"; Width = 1280; Height = 800 }
    "card-grid" = @{ Html = "websuite\cases\card-grid\index.html"; Width = 1280; Height = 800 }
    "css-selectors" = @{ Html = "websuite\cases\css-selectors\index.html"; Width = 800; Height = 1200 }
    "flex-positioning" = @{ Html = "websuite\cases\flex-positioning\index.html"; Width = 800; Height = 1000 }
    "form-elements" = @{ Html = "websuite\cases\form-elements\index.html"; Width = 800; Height = 600 }
    "gradient-backgrounds" = @{ Html = "websuite\cases\gradient-backgrounds\index.html"; Width = 800; Height = 600 }
    "image-gallery" = @{ Html = "websuite\cases\image-gallery\index.html"; Width = 1280; Height = 800 }
    "sticky-scroll" = @{ Html = "websuite\cases\sticky-scroll\index.html"; Width = 1280; Height = 800 }
}

Write-Host "=============================================="
Write-Host "Visual Test Runner"
Write-Host "Duration per case: ${Duration}ms"
Write-Host "=============================================="

# Build first
Write-Host ""
Write-Host "Building hiwave-smoke (release)..."
Push-Location $ProjectDir
try {
    & cargo build --release -p hiwave-smoke 2>&1 | Select-Object -Last 5
} finally {
    Pop-Location
}

$Passed = 0
$Failed = 0

if ($Case) {
    # Run single case
    if (-not $TestCases.ContainsKey($Case)) {
        Write-Host "Unknown case: $Case" -ForegroundColor Red
        Write-Host "Run with -Help to see available cases"
        exit 1
    }

    $tc = $TestCases[$Case]
    Run-Case -Name $Case -Html $tc.Html -Width $tc.Width -Height $tc.Height
} else {
    # Run all cases in order
    $CaseOrder = @(
        "new_tab", "about", "settings", "chrome_rustkit", "shelf",
        "article-typography", "card-grid", "css-selectors", "flex-positioning",
        "form-elements", "gradient-backgrounds", "image-gallery", "sticky-scroll"
    )

    foreach ($caseName in $CaseOrder) {
        $tc = $TestCases[$caseName]
        $result = Run-Case -Name $caseName -Html $tc.Html -Width $tc.Width -Height $tc.Height
        if ($result) {
            $Passed++
        } else {
            $Failed++
        }
    }

    Write-Host ""
    Write-Host "=============================================="
    Write-Host "Results: $Passed passed, $Failed failed"
    Write-Host "=============================================="
}
