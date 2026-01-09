#Requires -Version 5.1
<#
.SYNOPSIS
    Run non-gating live site smoke tests

.DESCRIPTION
    This script tests RustKit rendering against real websites.
    Results are collected but not gating - failures are expected as
    we work toward full web compatibility.

.PARAMETER OutputDir
    Directory to store results (default: websuite\live-site-results)

.PARAMETER Mode
    Capture mode: 'chromium-only', 'rustkit-only', or 'both' (default: chromium-only)

.EXAMPLE
    .\scripts\live_site_smoke.ps1
    .\scripts\live_site_smoke.ps1 -Mode both
    .\scripts\live_site_smoke.ps1 -Mode chromium-only -OutputDir "C:\results"
#>

param(
    [string]$OutputDir,
    [ValidateSet('chromium-only', 'rustkit-only', 'both')]
    [string]$Mode = 'chromium-only'
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$Config = Join-Path $ProjectDir "websuite\live-sites.json"
if (-not $OutputDir) {
    $OutputDir = Join-Path $ProjectDir "websuite\live-site-results"
}
$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$RunDir = Join-Path $OutputDir $Timestamp

Write-Host "Live Site Smoke Test"
Write-Host "===================="
Write-Host "Mode: $Mode"
Write-Host ""
Write-Host "WARNING: This is non-gating. Failures are expected during development."
Write-Host ""

# Check config exists
if (-not (Test-Path $Config)) {
    Write-Host "Error: Config not found at $Config" -ForegroundColor Red
    exit 1
}

# Create output directories
New-Item -ItemType Directory -Path "$RunDir\chromium" -Force | Out-Null
New-Item -ItemType Directory -Path "$RunDir\rustkit" -Force | Out-Null
New-Item -ItemType Directory -Path "$RunDir\diffs" -Force | Out-Null

# Build if needed (for RustKit HTTP support)
if ($Mode -ne 'chromium-only') {
    $HiWaveBin = Join-Path $ProjectDir "target\release\hiwave.exe"
    if (-not (Test-Path $HiWaveBin)) {
        Write-Host "Building HiWave..."
        Push-Location $ProjectDir
        try {
            & cargo build --release -p hiwave-app
        } catch {
            Write-Warning "Build failed, continuing anyway..."
        } finally {
            Pop-Location
        }
    }
}

$BaselineTool = Join-Path $ProjectDir "tools\websuite-baseline"

# ============================================================================
# Chromium Capture
# ============================================================================

if ($Mode -ne 'rustkit-only') {
    Write-Host "=== Chromium Capture ==="
    Write-Host ""

    Push-Location $BaselineTool
    try {
        if (-not (Test-Path "node_modules")) {
            Write-Host "Installing Playwright..."
            & npm install
            & npx playwright install chromium
        }
    } finally {
        Pop-Location
    }

    # Create capture script for live sites
    $CaptureScript = @'
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

async function main() {
    const configPath = process.argv[2];
    const outputDir = process.argv[3];

    const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

    const browser = await chromium.launch({ headless: true });
    const results = [];

    for (const site of config.sites) {
        console.log(`Capturing: ${site.name} (${site.url})`);

        try {
            const context = await browser.newContext({
                viewport: site.viewport,
                deviceScaleFactor: 2,
            });

            const page = await context.newPage();

            // Collect console logs
            const consoleLogs = [];
            page.on('console', msg => {
                consoleLogs.push({
                    type: msg.type(),
                    text: msg.text(),
                    time: new Date().toISOString()
                });
            });

            // Collect network requests
            const networkLogs = [];
            page.on('request', req => {
                networkLogs.push({
                    url: req.url(),
                    method: req.method(),
                    resourceType: req.resourceType(),
                    time: new Date().toISOString()
                });
            });

            const startTime = Date.now();

            await page.goto(site.url, {
                waitUntil: 'networkidle',
                timeout: config.output_config.timeout_ms
            });

            await page.waitForTimeout(site.wait_ms);

            const loadTime = Date.now() - startTime;

            // Capture screenshot
            const screenshotPath = path.join(outputDir, `${site.id}.png`);
            await page.screenshot({ path: screenshotPath, fullPage: false });

            await context.close();

            // Save logs
            fs.writeFileSync(
                path.join(outputDir, `${site.id}.console.json`),
                JSON.stringify(consoleLogs, null, 2)
            );
            fs.writeFileSync(
                path.join(outputDir, `${site.id}.network.json`),
                JSON.stringify(networkLogs, null, 2)
            );

            results.push({
                id: site.id,
                name: site.name,
                url: site.url,
                status: 'ok',
                load_time_ms: loadTime,
                console_log_count: consoleLogs.length,
                network_request_count: networkLogs.length
            });

            console.log(`  OK: ${loadTime}ms, ${networkLogs.length} requests`);

        } catch (error) {
            console.log(`  FAIL: ${error.message}`);
            results.push({
                id: site.id,
                name: site.name,
                url: site.url,
                status: 'error',
                error: error.message
            });
        }
    }

    await browser.close();

    // Write summary
    fs.writeFileSync(
        path.join(outputDir, 'summary.json'),
        JSON.stringify({
            timestamp: new Date().toISOString(),
            total: config.sites.length,
            passed: results.filter(r => r.status === 'ok').length,
            failed: results.filter(r => r.status !== 'ok').length,
            sites: results
        }, null, 2)
    );
}

main().catch(console.error);
'@

    $CaptureScriptPath = Join-Path $RunDir "capture_live.js"
    Set-Content -Path $CaptureScriptPath -Value $CaptureScript

    # Run Chromium capture
    Push-Location $BaselineTool
    try {
        & node $CaptureScriptPath $Config "$RunDir\chromium"
    } finally {
        Pop-Location
    }

    # Copy summary to root with chromium prefix
    if (Test-Path "$RunDir\chromium\summary.json") {
        Move-Item "$RunDir\chromium\summary.json" "$RunDir\chromium_summary.json"
    }
}

# ============================================================================
# RustKit Capture (when HTTP(S) support is available)
# ============================================================================

if ($Mode -ne 'chromium-only') {
    Write-Host ""
    Write-Host "=== RustKit Capture ==="
    Write-Host ""

    $SmokeBin = Join-Path $ProjectDir "target\release\hiwave-smoke.exe"

    if (-not (Test-Path $SmokeBin)) {
        Write-Host "Building hiwave-smoke..."
        Push-Location $ProjectDir
        try {
            & cargo build -p hiwave-smoke --release
        } finally {
            Pop-Location
        }
    }

    # Note: RustKit live site capture requires HTTP(S) network support
    # which is not yet fully implemented. For now, we skip this step.
    Write-Host "NOTICE: RustKit live site capture requires HTTP(S) network support."
    Write-Host "        This feature is in development. Skipping RustKit capture."
    Write-Host ""

    # Create placeholder summary using Python
    $pythonScript = @"
import json
import os
from datetime import datetime

config_path = r'$Config'
output_dir = r'$RunDir\rustkit'

with open(config_path) as f:
    config = json.load(f)

summary = {
    "timestamp": datetime.now().isoformat(),
    "renderer": "rustkit",
    "status": "skipped",
    "reason": "HTTP(S) network support not yet implemented",
    "total": len(config.get("sites", [])),
    "passed": 0,
    "failed": 0,
    "skipped": len(config.get("sites", [])),
    "sites": [
        {
            "id": site["id"],
            "name": site["name"],
            "url": site["url"],
            "status": "skipped",
            "reason": "HTTP(S) not implemented"
        }
        for site in config.get("sites", [])
    ]
}

with open(os.path.join(output_dir, "summary.json"), "w") as f:
    json.dump(summary, f, indent=2)

print("RustKit summary created (skipped)")
"@

    & python -c $pythonScript

    if (Test-Path "$RunDir\rustkit\summary.json") {
        Move-Item "$RunDir\rustkit\summary.json" "$RunDir\rustkit_summary.json"
    }
}

# ============================================================================
# Comparison (when both captures are available)
# ============================================================================

if ($Mode -eq 'both') {
    Write-Host ""
    Write-Host "=== Comparison ==="
    Write-Host ""

    Write-Host "NOTICE: Comparison requires both Chromium and RustKit captures."
    Write-Host "        RustKit capture is currently skipped."
    Write-Host ""
}

# ============================================================================
# Final Summary
# ============================================================================

Write-Host ""
Write-Host "Live Site Smoke Complete"
Write-Host "========================"
Write-Host "Results: $RunDir"
Write-Host ""

# Print Chromium summary if available
if (Test-Path "$RunDir\chromium_summary.json") {
    Write-Host "Chromium Results:"
    $summary = Get-Content "$RunDir\chromium_summary.json" | ConvertFrom-Json
    Write-Host "  Total:  $($summary.total)"
    Write-Host "  Passed: $($summary.passed)"
    Write-Host "  Failed: $($summary.failed)"
    Write-Host ""
    Write-Host "  Sites:"
    foreach ($site in $summary.sites) {
        $status = if ($site.status -eq 'ok') { "[OK]" } else { "[FAIL]" }
        $color = if ($site.status -eq 'ok') { "Green" } else { "Red" }
        Write-Host "    $status $($site.name)" -ForegroundColor $color
    }
}

# Print RustKit summary if available
if (Test-Path "$RunDir\rustkit_summary.json") {
    Write-Host ""
    Write-Host "RustKit Results:"
    $summary = Get-Content "$RunDir\rustkit_summary.json" | ConvertFrom-Json
    if ($summary.status -eq 'skipped') {
        Write-Host "  Status: Skipped ($($summary.reason))"
    } else {
        Write-Host "  Total:  $($summary.total)"
        Write-Host "  Passed: $($summary.passed)"
        Write-Host "  Failed: $($summary.failed)"
    }
}
