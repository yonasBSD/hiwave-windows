# capture_goldens.ps1 - Capture golden images for all test fixtures
#
# Usage: .\scripts\capture_goldens.ps1 [fixture_name]
#        .\scripts\capture_goldens.ps1 -Width 1280 -Height 720 [fixture_name]
#
# If no fixture_name is provided, captures all fixtures.
# Use standardized sizes for deterministic comparison.

param(
    [int]$Width = 800,
    [int]$Height = 600,
    [int]$DurationMs = 1000,
    [string]$Fixture = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$FixturesDir = Join-Path $ProjectRoot "fixtures"
$GoldensDir = Join-Path $ProjectRoot "goldens"

# Create goldens directory if it doesn't exist
New-Item -ItemType Directory -Force -Path $GoldensDir | Out-Null

# Build release first
Write-Host "Building release..." -ForegroundColor Cyan
Push-Location $ProjectRoot
try {
    cargo build --release -p hiwave-smoke 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Warning: release build failed, trying dev build..." -ForegroundColor Yellow
        cargo build -p hiwave-smoke
    }
} finally {
    Pop-Location
}

$SmokeBin = Join-Path $ProjectRoot "target\release\hiwave-smoke.exe"
if (-not (Test-Path $SmokeBin)) {
    $SmokeBin = Join-Path $ProjectRoot "target\debug\hiwave-smoke.exe"
}

if (-not (Test-Path $SmokeBin)) {
    Write-Host "ERROR: hiwave-smoke binary not found" -ForegroundColor Red
    exit 1
}

Write-Host "Capture settings: ${Width}x${Height}, duration=${DurationMs}ms" -ForegroundColor Gray
Write-Host ""

function Capture-Fixture {
    param([string]$FixturePath)
    
    $fixtureName = [System.IO.Path]::GetFileNameWithoutExtension($FixturePath)
    $outputPath = Join-Path $GoldensDir "${fixtureName}.ppm"
    
    Write-Host "Capturing: $fixtureName (${Width}x${Height})" -ForegroundColor White
    
    # Run smoke harness with the fixture
    $process = Start-Process -FilePath $SmokeBin -ArgumentList @(
        "--html-file", $FixturePath,
        "--width", $Width,
        "--height", $Height,
        "--duration-ms", $DurationMs,
        "--dump-frame", $outputPath
    ) -NoNewWindow -Wait -PassThru
    
    if ($process.ExitCode -ne 0 -or -not (Test-Path $outputPath)) {
        Write-Host "  Warning: Capture failed for $fixtureName" -ForegroundColor Yellow
        return $false
    }
    
    $size = (Get-Item $outputPath).Length
    $sizeKB = [math]::Round($size / 1024, 1)
    Write-Host "  Captured: $outputPath (${sizeKB}KB)" -ForegroundColor Green
    
    # Save metadata
    $metadata = @{
        fixture = $fixtureName
        width = $Width
        height = $Height
        duration_ms = $DurationMs
        captured_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    } | ConvertTo-Json
    
    $metaPath = Join-Path $GoldensDir "${fixtureName}.meta.json"
    $metadata | Out-File -FilePath $metaPath -Encoding utf8
    
    return $true
}

# Main execution
$captured = 0
$failed = 0

if ($Fixture) {
    # Capture specific fixture
    $fixturePath = Join-Path $FixturesDir $Fixture
    if (-not (Test-Path $fixturePath)) {
        $fixturePath = Join-Path $FixturesDir "${Fixture}.html"
    }
    
    if (Test-Path $fixturePath) {
        if (Capture-Fixture -FixturePath $fixturePath) {
            $captured++
        } else {
            $failed++
        }
    } else {
        Write-Host "Fixture not found: $Fixture" -ForegroundColor Red
        exit 1
    }
} else {
    # Capture all fixtures
    Write-Host "Capturing golden images for all fixtures..." -ForegroundColor Cyan
    Write-Host ""
    
    Get-ChildItem -Path $FixturesDir -Filter "*.html" | ForEach-Object {
        if (Capture-Fixture -FixturePath $_.FullName) {
            $captured++
        } else {
            $failed++
        }
        Write-Host ""
    }
}

Write-Host "==============================" -ForegroundColor Cyan
Write-Host "Summary: $captured captured, $failed failed"
Write-Host ""
Write-Host "Golden images stored in: $GoldensDir"


