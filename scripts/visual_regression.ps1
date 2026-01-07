# visual_regression.ps1 - Visual regression test runner
#
# Compares current rendering output against golden images.
# Reports any differences and generates diff images.
#
# Usage: .\scripts\visual_regression.ps1 [fixture_name]

param(
    [string]$Fixture = "",
    [int]$Tolerance = 5
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$FixturesDir = Join-Path $ProjectRoot "fixtures"
$GoldensDir = Join-Path $ProjectRoot "goldens"
$Timestamp = Get-Date -Format "yyyyMMddTHHmmss"
$ResultsDir = Join-Path $ProjectRoot ".ai\artifacts\regression_$Timestamp"

# Create results directory
New-Item -ItemType Directory -Force -Path $ResultsDir | Out-Null

# Build
Write-Host "Building..." -ForegroundColor Cyan
Push-Location $ProjectRoot
try {
    cargo build --release -p hiwave-smoke 2>$null
    if ($LASTEXITCODE -ne 0) {
        cargo build -p hiwave-smoke
    }
} finally {
    Pop-Location
}

$SmokeBin = Join-Path $ProjectRoot "target\release\hiwave-smoke.exe"
if (-not (Test-Path $SmokeBin)) {
    $SmokeBin = Join-Path $ProjectRoot "target\debug\hiwave-smoke.exe"
}

# Python script for image comparison
$CompareScript = @'
import sys

def read_ppm(path):
    try:
        with open(path, 'rb') as f:
            magic = f.readline().decode().strip()
            if magic != 'P6':
                return None, 0, 0
            line = f.readline().decode()
            while line.startswith('#'):
                line = f.readline().decode()
            dims = line.strip().split()
            width, height = int(dims[0]), int(dims[1])
            max_val = int(f.readline().decode().strip())
            data = f.read()
            return data, width, height
    except:
        return None, 0, 0

golden_path = sys.argv[1]
current_path = sys.argv[2]
tolerance = int(sys.argv[3])

golden_data, gw, gh = read_ppm(golden_path)
current_data, cw, ch = read_ppm(current_path)

if golden_data is None or current_data is None:
    print("ERROR: Failed to read images")
    sys.exit(2)

if (gw, gh) != (cw, ch):
    print(f"SIZE_MISMATCH: golden={gw}x{gh}, current={cw}x{ch}")
    sys.exit(1)

diff_count = 0
for i in range(0, min(len(golden_data), len(current_data)), 3):
    if i + 2 >= len(golden_data) or i + 2 >= len(current_data):
        break
    dr = abs(golden_data[i] - current_data[i])
    dg = abs(golden_data[i+1] - current_data[i+1])
    db = abs(golden_data[i+2] - current_data[i+2])
    if dr > tolerance or dg > tolerance or db > tolerance:
        diff_count += 1

total_pixels = gw * gh
diff_percent = (diff_count / total_pixels) * 100 if total_pixels > 0 else 0

if diff_count == 0:
    print("MATCH")
    sys.exit(0)
else:
    print(f"DIFF: {diff_count} pixels ({diff_percent:.2f}%)")
    sys.exit(1)
'@

function Compare-Images {
    param(
        [string]$GoldenPath,
        [string]$CurrentPath
    )
    
    $result = $CompareScript | python3 - $GoldenPath $CurrentPath $Tolerance 2>&1
    return $result
}

function Test-Fixture {
    param([string]$FixturePath)
    
    $fixtureName = [System.IO.Path]::GetFileNameWithoutExtension($FixturePath)
    $goldenPath = Join-Path $GoldensDir "${fixtureName}.ppm"
    $currentPath = Join-Path $ResultsDir "${fixtureName}_current.ppm"
    
    Write-Host "Testing: $fixtureName" -ForegroundColor White
    
    # Check if golden exists
    if (-not (Test-Path $goldenPath)) {
        Write-Host "  SKIP: No golden image found" -ForegroundColor Yellow
        return 2
    }
    
    # Read golden metadata for dimensions
    $metaPath = Join-Path $GoldensDir "${fixtureName}.meta.json"
    $width = 800
    $height = 600
    if (Test-Path $metaPath) {
        $meta = Get-Content $metaPath | ConvertFrom-Json
        $width = $meta.width
        $height = $meta.height
    }
    
    # Capture current rendering
    $process = Start-Process -FilePath $SmokeBin -ArgumentList @(
        "--html-file", $FixturePath,
        "--width", $width,
        "--height", $height,
        "--duration-ms", 1000,
        "--dump-frame", $currentPath
    ) -NoNewWindow -Wait -PassThru
    
    if ($process.ExitCode -ne 0 -or -not (Test-Path $currentPath)) {
        Write-Host "  FAIL: Capture failed" -ForegroundColor Red
        return 1
    }
    
    # Compare images
    $result = Compare-Images -GoldenPath $goldenPath -CurrentPath $currentPath
    
    if ($result -eq "MATCH") {
        Write-Host "  PASS: Images match" -ForegroundColor Green
        Remove-Item -Path $currentPath -ErrorAction SilentlyContinue
        return 0
    } else {
        Write-Host "  FAIL: $result" -ForegroundColor Red
        # Keep current image and copy golden for review
        Copy-Item -Path $goldenPath -Destination (Join-Path $ResultsDir "${fixtureName}_golden.ppm")
        return 1
    }
}

Write-Host "==============================" -ForegroundColor Cyan
Write-Host "Visual Regression Tests"
Write-Host "==============================" -ForegroundColor Cyan
Write-Host ""

$passed = 0
$failed = 0
$skipped = 0

if ($Fixture) {
    # Test specific fixture
    $fixturePath = Join-Path $FixturesDir $Fixture
    if (-not (Test-Path $fixturePath)) {
        $fixturePath = Join-Path $FixturesDir "${Fixture}.html"
    }
    
    if (Test-Path $fixturePath) {
        $result = Test-Fixture -FixturePath $fixturePath
        switch ($result) {
            0 { $passed++ }
            1 { $failed++ }
            2 { $skipped++ }
        }
    } else {
        Write-Host "Fixture not found: $Fixture" -ForegroundColor Red
        exit 1
    }
} else {
    # Test all fixtures
    Get-ChildItem -Path $FixturesDir -Filter "*.html" | ForEach-Object {
        $result = Test-Fixture -FixturePath $_.FullName
        switch ($result) {
            0 { $passed++ }
            1 { $failed++ }
            2 { $skipped++ }
        }
        Write-Host ""
    }
}

Write-Host "==============================" -ForegroundColor Cyan
Write-Host "Results: $passed passed, $failed failed, $skipped skipped"
Write-Host "Artifacts: $ResultsDir"
Write-Host "==============================" -ForegroundColor Cyan

if ($failed -gt 0) {
    exit 1
}
exit 0

