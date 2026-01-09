#Requires -Version 5.1
<#
.SYNOPSIS
    Local wrapper for ai-orchestrator

.DESCRIPTION
    The orchestrator itself is gitignored; this script provides a tracked entry point.

.EXAMPLE
    .\scripts\aiorch_local.ps1 canary run --profile release --duration-ms 5000 --dump-frame
    .\scripts\aiorch_local.ps1 verify <work_order_id>
    .\scripts\aiorch_local.ps1 status
#>

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir

$AiorchPath = if ($env:AIORCH_PATH) {
    $env:AIORCH_PATH
} else {
    Join-Path $RepoRoot "tools\ai-orchestrator\aiorch.py"
}

if (-not (Test-Path $AiorchPath)) {
    Write-Host "ERROR: ai-orchestrator not found at $AiorchPath" -ForegroundColor Red
    Write-Host ""
    Write-Host "The ai-orchestrator is not tracked in this repository."
    Write-Host "To use it, copy or symlink your local orchestrator to:"
    Write-Host "  $RepoRoot\tools\ai-orchestrator\"
    Write-Host ""
    Write-Host "Or set AIORCH_PATH to point to your aiorch.py:"
    Write-Host "  `$env:AIORCH_PATH = 'C:\path\to\your\aiorch.py'"
    Write-Host "  .\scripts\aiorch_local.ps1 $($Arguments -join ' ')"
    exit 1
}

Push-Location $RepoRoot
try {
    & python $AiorchPath @Arguments
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}
