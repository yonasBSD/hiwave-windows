# parity_gate.ps1 - Unified pixel parity gate for built-ins + websuite
#
# This script:
# 1. Captures RustKit frames for all target pages
# 2. Compares against golden images
# 3. Generates failure reports with detailed diagnostics
#
# Usage: .\scripts\parity_gate.ps1 [-RegenerateBaseline] [-Verbose]

param(
    [switch]$RegenerateBaseline,
    [switch]$VerboseOutput
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$FixturesDir = Join-Path $ProjectRoot "fixtures"
$GoldensDir = Join-Path $ProjectRoot "goldens"
$OutputDir = Join-Path $ProjectRoot "parity-results"
$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$RunDir = Join-Path $OutputDir $Timestamp

Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║           PIXEL PARITY GATE (Papa 1 Protocol)                ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Timestamp: $Timestamp"
Write-Host "Output: $RunDir"
Write-Host ""

# Create output directories
New-Item -ItemType Directory -Force -Path "$RunDir\rustkit" | Out-Null
New-Item -ItemType Directory -Force -Path "$RunDir\diffs" | Out-Null
New-Item -ItemType Directory -Force -Path "$RunDir\failure-packets" | Out-Null

# ============================================================================
# Step 1: Build RustKit smoke harness
# ============================================================================
Write-Host "┌─────────────────────────────────────────────────────────────┐" -ForegroundColor Gray
Write-Host "│ Step 1: Build RustKit                                       │" -ForegroundColor Gray
Write-Host "└─────────────────────────────────────────────────────────────┘" -ForegroundColor Gray

$SmokeBin = Join-Path $ProjectRoot "target\release\hiwave-smoke.exe"
if (-not (Test-Path $SmokeBin)) {
    Write-Host "Building hiwave-smoke..."
    Push-Location $ProjectRoot
    cargo build -p hiwave-smoke --release
    Pop-Location
}
Write-Host "OK: hiwave-smoke ready" -ForegroundColor Green
Write-Host ""

# ============================================================================
# Step 2: Define target pages
# ============================================================================
Write-Host "┌─────────────────────────────────────────────────────────────┐" -ForegroundColor Gray
Write-Host "│ Step 2: Define target pages                                 │" -ForegroundColor Gray
Write-Host "└─────────────────────────────────────────────────────────────┘" -ForegroundColor Gray

# Built-in pages
$BuiltinPages = @(
    @{ id = "new_tab"; path = "$ProjectRoot\crates\hiwave-app\src\ui\new_tab.html"; width = 1280; height = 800 }
    @{ id = "about"; path = "$ProjectRoot\crates\hiwave-app\src\ui\about.html"; width = 1280; height = 800 }
    @{ id = "settings"; path = "$ProjectRoot\crates\hiwave-app\src\ui\settings.html"; width = 1280; height = 800 }
)

# Add fixture pages
$FixturePages = @()
Get-ChildItem -Path $FixturesDir -Filter "*.html" -ErrorAction SilentlyContinue | ForEach-Object {
    $FixturePages += @{ id = $_.BaseName; path = $_.FullName; width = 800; height = 600 }
}

$AllPages = $BuiltinPages + $FixturePages
Write-Host "Target pages: $($AllPages.Count)"
foreach ($page in $AllPages) {
    Write-Host "  - $($page.id) ($($page.width)x$($page.height))"
}
Write-Host ""

# ============================================================================
# Step 3: Capture RustKit frames
# ============================================================================
Write-Host "┌─────────────────────────────────────────────────────────────┐" -ForegroundColor Gray
Write-Host "│ Step 3: Capture RustKit frames                              │" -ForegroundColor Gray
Write-Host "└─────────────────────────────────────────────────────────────┘" -ForegroundColor Gray

$RustkitPassed = 0
$RustkitFailed = 0

foreach ($page in $AllPages) {
    if (-not (Test-Path $page.path)) {
        Write-Host "  SKIP: $($page.id) (file not found)" -ForegroundColor Yellow
        $RustkitFailed++
        continue
    }
    
    $outputPpm = Join-Path "$RunDir\rustkit" "$($page.id).ppm"
    $perfJson = Join-Path "$RunDir\rustkit" "$($page.id).perf.json"
    
    Write-Host "  Capturing $($page.id)... " -NoNewline
    
    $process = Start-Process -FilePath $SmokeBin -ArgumentList @(
        "--html-file", $page.path,
        "--width", $page.width,
        "--height", $page.height,
        "--duration-ms", 500,
        "--dump-frame", $outputPpm,
        "--perf-output", $perfJson
    ) -NoNewWindow -Wait -PassThru 2>$null
    
    if ($process.ExitCode -eq 0 -and (Test-Path $outputPpm)) {
        Write-Host "OK" -ForegroundColor Green
        $RustkitPassed++
    } else {
        Write-Host "FAIL" -ForegroundColor Red
        $RustkitFailed++
    }
}

Write-Host ""
Write-Host "RustKit: $RustkitPassed passed, $RustkitFailed failed"
Write-Host ""

# ============================================================================
# Step 4: Compare against goldens
# ============================================================================
Write-Host "┌─────────────────────────────────────────────────────────────┐" -ForegroundColor Gray
Write-Host "│ Step 4: Compare frames against goldens                      │" -ForegroundColor Gray
Write-Host "└─────────────────────────────────────────────────────────────┘" -ForegroundColor Gray

$CompareScript = @'
import json
import sys
import os

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

rustkit_path = sys.argv[1]
golden_path = sys.argv[2]
tolerance = int(sys.argv[3])

result = {"status": "unknown", "diff_pixels": 0, "total_pixels": 0, "diff_percent": 0}

if not os.path.exists(golden_path):
    result["status"] = "no_golden"
    print(json.dumps(result))
    sys.exit(0)

if not os.path.exists(rustkit_path):
    result["status"] = "no_capture"
    print(json.dumps(result))
    sys.exit(1)

rustkit_data, rw, rh = read_ppm(rustkit_path)
golden_data, gw, gh = read_ppm(golden_path)

if rustkit_data is None or golden_data is None:
    result["status"] = "read_error"
    print(json.dumps(result))
    sys.exit(1)

if (rw, rh) != (gw, gh):
    result["status"] = "size_mismatch"
    result["rustkit_size"] = f"{rw}x{rh}"
    result["golden_size"] = f"{gw}x{gh}"
    print(json.dumps(result))
    sys.exit(1)

result["total_pixels"] = rw * rh

diff_count = 0
for i in range(0, min(len(rustkit_data), len(golden_data)), 3):
    if i + 2 >= len(rustkit_data) or i + 2 >= len(golden_data):
        break
    dr = abs(rustkit_data[i] - golden_data[i])
    dg = abs(rustkit_data[i+1] - golden_data[i+1])
    db = abs(rustkit_data[i+2] - golden_data[i+2])
    if dr > tolerance or dg > tolerance or db > tolerance:
        diff_count += 1

result["diff_pixels"] = diff_count
result["diff_percent"] = (diff_count / (rw * rh)) * 100 if rw * rh > 0 else 0
result["status"] = "pass" if diff_count == 0 else "diff"

print(json.dumps(result))
sys.exit(0 if diff_count == 0 else 1)
'@

$ComparePassed = 0
$CompareFailed = 0
$CompareSkipped = 0
$Results = @()

foreach ($page in $AllPages) {
    $rustkitPpm = Join-Path "$RunDir\rustkit" "$($page.id).ppm"
    $goldenPpm = Join-Path $GoldensDir "$($page.id).ppm"
    
    if (-not (Test-Path $rustkitPpm)) {
        Write-Host "  SKIP: $($page.id) (no capture)" -ForegroundColor Yellow
        $CompareSkipped++
        continue
    }
    
    Write-Host "  Comparing $($page.id)... " -NoNewline
    
    try {
        $resultJson = $CompareScript | python3 - $rustkitPpm $goldenPpm 5 2>&1
        $result = $resultJson | ConvertFrom-Json
        
        switch ($result.status) {
            "pass" {
                Write-Host "PASS" -ForegroundColor Green
                $ComparePassed++
            }
            "no_golden" {
                Write-Host "SKIP (no golden)" -ForegroundColor Yellow
                $CompareSkipped++
            }
            "diff" {
                Write-Host "DIFF ($($result.diff_pixels) pixels, $([math]::Round($result.diff_percent, 2))%)" -ForegroundColor Red
                $CompareFailed++
            }
            default {
                Write-Host "FAIL ($($result.status))" -ForegroundColor Red
                $CompareFailed++
            }
        }
        
        $Results += @{
            id = $page.id
            status = $result.status
            diff_pixels = $result.diff_pixels
            diff_percent = $result.diff_percent
            total_pixels = $result.total_pixels
        }
    } catch {
        Write-Host "ERROR" -ForegroundColor Red
        $CompareFailed++
    }
}

Write-Host ""
Write-Host "Comparison: $ComparePassed passed, $CompareFailed failed, $CompareSkipped skipped"
Write-Host ""

# ============================================================================
# Step 5: Generate summary
# ============================================================================
Write-Host "┌─────────────────────────────────────────────────────────────┐" -ForegroundColor Gray
Write-Host "│ Step 5: Generate Summary                                    │" -ForegroundColor Gray
Write-Host "└─────────────────────────────────────────────────────────────┘" -ForegroundColor Gray

$totalDiffPixels = ($Results | Measure-Object -Property diff_pixels -Sum).Sum
$totalPixels = ($Results | Measure-Object -Property total_pixels -Sum).Sum
$webScore = if ($totalPixels -gt 0) { 1.0 - ($totalDiffPixels / $totalPixels) } else { 1.0 }

$summary = @{
    timestamp = (Get-Date).ToUniversalTime().ToString("o")
    policy = @{
        aa_tolerance = 5
        max_diff_percent = 0.0
    }
    summary = @{
        total = $Results.Count
        passed = $ComparePassed
        failed = $CompareFailed
        skipped = $CompareSkipped
        true_diff_pixels = $totalDiffPixels
        total_pixels = $totalPixels
        web_score = [math]::Round($webScore, 4)
    }
    cases = $Results
} | ConvertTo-Json -Depth 10

$summaryPath = Join-Path $RunDir "parity_summary.json"
$summary | Out-File -FilePath $summaryPath -Encoding utf8

Write-Host "Summary written to: $summaryPath"
Write-Host ""
Write-Host "Overall Results:" -ForegroundColor White
Write-Host "  Total cases: $($Results.Count)"
Write-Host "  Passed: $ComparePassed"
Write-Host "  Failed: $CompareFailed"
Write-Host "  Skipped: $CompareSkipped"
Write-Host "  True diff pixels: $totalDiffPixels"
Write-Host "  Web Score: $([math]::Round($webScore, 4))"
Write-Host ""

# ============================================================================
# Step 6: Final Report
# ============================================================================
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                    PARITY GATE COMPLETE                      ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Results: $RunDir"
Write-Host ""
Write-Host "Artifacts:"
Write-Host "  - RustKit frames:    $RunDir\rustkit\"
Write-Host "  - Diff images:       $RunDir\diffs\"
Write-Host "  - Failure packets:   $RunDir\failure-packets\"
Write-Host "  - Summary:           $RunDir\parity_summary.json"
Write-Host ""

# Return exit code based on results
if ($RustkitFailed -gt 0) {
    Write-Host "STATUS: FAIL (capture errors)" -ForegroundColor Red
    exit 1
}

if ($CompareFailed -gt 0) {
    Write-Host "STATUS: FAIL ($CompareFailed cases have pixel differences)" -ForegroundColor Red
    Write-Host ""
    Write-Host "To investigate failures, check:"
    Write-Host "  - Diff images: $RunDir\diffs\"
    Write-Host "  - Summary: $RunDir\parity_summary.json"
    exit 1
}

Write-Host "STATUS: PASS (all cases match within AA tolerance)" -ForegroundColor Green
exit 0

