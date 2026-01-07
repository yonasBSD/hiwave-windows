# canary_golden_gate.ps1 - Golden image diff gate for canary runs
#
# Produces a JSON diff report comparing current render against golden.
# Exit code 0 = pass (diff within tolerance), 1 = fail (regression)
#
# Usage:
#   .\scripts\canary_golden_gate.ps1 -Fixture typography -OutputFile C:\temp\diff_report.json
#   .\scripts\canary_golden_gate.ps1 -All -OutputFile C:\temp\all_diffs.json

param(
    [string]$Fixture = "",
    [switch]$All,
    [string]$OutputFile = "",
    [int]$Tolerance = 5,
    [int]$Width = 800,
    [int]$Height = 600,
    [int]$DurationMs = 1000
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$FixturesDir = Join-Path $ProjectRoot "fixtures"
$GoldensDir = Join-Path $ProjectRoot "goldens"
$TempDir = Join-Path $env:TEMP "hiwave_canary_$(Get-Random)"

New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

# Ensure smoke binary exists
$SmokeBin = Join-Path $ProjectRoot "target\release\hiwave-smoke.exe"
if (-not (Test-Path $SmokeBin)) {
    Write-Host "Building hiwave-smoke..." -ForegroundColor Cyan
    Push-Location $ProjectRoot
    cargo build -p hiwave-smoke --release 2>$null
    if ($LASTEXITCODE -ne 0) {
        $SmokeBin = Join-Path $ProjectRoot "target\debug\hiwave-smoke.exe"
        cargo build -p hiwave-smoke
    }
    Pop-Location
}

# Python script for comparison and JSON output
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

fixture_name = sys.argv[1]
golden_path = sys.argv[2]
current_path = sys.argv[3]
tolerance = int(sys.argv[4])

result = {
    "fixture": fixture_name,
    "golden_path": golden_path,
    "current_path": current_path,
    "status": "unknown",
    "diff_pixels": 0,
    "total_pixels": 0,
    "diff_percent": 0.0,
    "tolerance": tolerance,
    "dimensions": {"width": 0, "height": 0},
    "error": None
}

if not os.path.exists(golden_path):
    result["status"] = "no_golden"
    result["error"] = f"Golden image not found: {golden_path}"
    print(json.dumps(result))
    sys.exit(0)

if not os.path.exists(current_path):
    result["status"] = "capture_failed"
    result["error"] = f"Current capture not found: {current_path}"
    print(json.dumps(result))
    sys.exit(1)

golden_data, gw, gh = read_ppm(golden_path)
current_data, cw, ch = read_ppm(current_path)

if golden_data is None:
    result["status"] = "invalid_golden"
    result["error"] = "Failed to read golden PPM"
    print(json.dumps(result))
    sys.exit(1)

if current_data is None:
    result["status"] = "invalid_current"
    result["error"] = "Failed to read current PPM"
    print(json.dumps(result))
    sys.exit(1)

if (gw, gh) != (cw, ch):
    result["status"] = "size_mismatch"
    result["error"] = f"Size mismatch: golden={gw}x{gh}, current={cw}x{ch}"
    result["dimensions"] = {"golden": {"width": gw, "height": gh}, "current": {"width": cw, "height": ch}}
    print(json.dumps(result))
    sys.exit(1)

result["dimensions"] = {"width": gw, "height": gh}
result["total_pixels"] = gw * gh

diff_count = 0
for i in range(0, min(len(golden_data), len(current_data)), 3):
    if i + 2 >= len(golden_data) or i + 2 >= len(current_data):
        break
    dr = abs(golden_data[i] - current_data[i])
    dg = abs(golden_data[i+1] - current_data[i+1])
    db = abs(golden_data[i+2] - current_data[i+2])
    if dr > tolerance or dg > tolerance or db > tolerance:
        diff_count += 1

result["diff_pixels"] = diff_count
result["diff_percent"] = (diff_count / (gw * gh)) * 100 if gw * gh > 0 else 0

if diff_count == 0:
    result["status"] = "pass"
else:
    result["status"] = "diff"

print(json.dumps(result, indent=2))
sys.exit(0 if diff_count == 0 else 1)
'@

function Run-FixtureTest {
    param([string]$FixtureName)
    
    $fixturePath = Join-Path $FixturesDir "${FixtureName}.html"
    $goldenPath = Join-Path $GoldensDir "${FixtureName}.ppm"
    $currentPath = Join-Path $TempDir "${FixtureName}_current.ppm"
    
    # Capture current frame
    $null = Start-Process -FilePath $SmokeBin -ArgumentList @(
        "--html-file", $fixturePath,
        "--width", $Width,
        "--height", $Height,
        "--duration-ms", $DurationMs,
        "--dump-frame", $currentPath
    ) -NoNewWindow -Wait -PassThru 2>$null
    
    # Compare and report
    $result = $CompareScript | python3 - $FixtureName $goldenPath $currentPath $Tolerance 2>&1
    return $result
}

# Main execution
if ($Fixture) {
    # Single fixture test
    $result = Run-FixtureTest -FixtureName $Fixture
    
    if ($OutputFile) {
        $result | Out-File -FilePath $OutputFile -Encoding utf8
    } else {
        Write-Output $result
    }
    
    # Check status for exit code
    $status = ($result | ConvertFrom-Json).status
    if ($status -eq "pass") { exit 0 } else { exit 1 }

} elseif ($All) {
    # Run all fixtures
    $results = @()
    $overallPass = $true
    
    Get-ChildItem -Path $FixturesDir -Filter "*.html" | ForEach-Object {
        $fixtureName = $_.BaseName
        try {
            $result = Run-FixtureTest -FixtureName $fixtureName
            $resultObj = $result | ConvertFrom-Json
            $results += $resultObj
            
            if ($resultObj.status -ne "pass" -and $resultObj.status -ne "no_golden") {
                $overallPass = $false
            }
        } catch {
            $results += @{ status = "error"; fixture = $fixtureName }
            $overallPass = $false
        }
    }
    
    # Create summary report
    $passed = ($results | Where-Object { $_.status -eq "pass" }).Count
    $failed = ($results | Where-Object { $_.status -in @("diff", "error", "capture_failed") }).Count
    $noGolden = ($results | Where-Object { $_.status -eq "no_golden" }).Count
    
    $summary = @{
        summary = @{
            total = $results.Count
            passed = $passed
            failed = $failed
            no_golden = $noGolden
            status = if ($failed -eq 0) { "pass" } else { "fail" }
        }
        capture_config = @{
            width = $Width
            height = $Height
            tolerance = $Tolerance
        }
        fixtures = $results
    } | ConvertTo-Json -Depth 10
    
    if ($OutputFile) {
        $summary | Out-File -FilePath $OutputFile -Encoding utf8
    } else {
        Write-Output $summary
    }
    
    if ($overallPass) { exit 0 } else { exit 1 }

} else {
    Write-Host "Usage: $($MyInvocation.MyCommand.Name) -Fixture <name> [-OutputFile <file>]" -ForegroundColor Yellow
    Write-Host "       $($MyInvocation.MyCommand.Name) -All [-OutputFile <file>]" -ForegroundColor Yellow
    exit 1
}

# Cleanup
Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue


