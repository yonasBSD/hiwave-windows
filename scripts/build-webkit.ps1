<# 
.SYNOPSIS
    Build WebKit WinCairo for HiWave development.

.DESCRIPTION
    This script automates the WebKit WinCairo build process for use with HiWave.
    It handles environment setup, dependency checking, and build configuration.

.PARAMETER Config
    Build configuration: Debug or Release (default: Release)

.PARAMETER Target
    Build target: All, MiniBrowser, WebKit2 (default: All)

.PARAMETER WebKitRoot
    Path to WebKit source (default: P:\WebKit)

.PARAMETER Clean
    Perform a clean build

.PARAMETER Jobs
    Number of parallel build jobs (default: auto-detect)

.EXAMPLE
    .\build-webkit.ps1 -Config Release -Target MiniBrowser

.EXAMPLE
    .\build-webkit.ps1 -Clean -Jobs 8
#>

param(
    [ValidateSet("Debug", "Release")]
    [string]$Config = "Release",
    
    [ValidateSet("All", "MiniBrowser", "WebKit2", "WebCore")]
    [string]$Target = "All",
    
    [string]$WebKitRoot = "P:\WebKit",
    
    [switch]$Clean,
    
    [int]$Jobs = 0
)

$ErrorActionPreference = "Stop"

# ============================================================================
# Configuration
# ============================================================================

$BuildDir = Join-Path $WebKitRoot "WebKitBuild\$Config"
$DepotToolsUrl = "https://chromium.googlesource.com/chromium/tools/depot_tools.git"

# ============================================================================
# Utilities
# ============================================================================

function Write-Status {
    param([string]$Message)
    Write-Host "[WebKit Build] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[WebKit Build] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WebKit Build] WARNING: $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[WebKit Build] ERROR: $Message" -ForegroundColor Red
}

function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

# ============================================================================
# Prerequisite Checks
# ============================================================================

function Test-Prerequisites {
    Write-Status "Checking prerequisites..."
    
    $missing = @()
    
    # Visual Studio
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vsWhere)) {
        $missing += "Visual Studio 2022"
    } else {
        $vsPath = & $vsWhere -latest -property installationPath
        if (-not $vsPath) {
            $missing += "Visual Studio 2022"
        }
    }
    
    # CMake
    if (-not (Test-Command "cmake")) {
        $missing += "CMake"
    }
    
    # Python
    if (-not (Test-Command "python")) {
        $missing += "Python 3"
    }
    
    # Ruby
    if (-not (Test-Command "ruby")) {
        $missing += "Ruby"
    }
    
    # Perl
    if (-not (Test-Command "perl")) {
        $missing += "Perl"
    }
    
    # Git
    if (-not (Test-Command "git")) {
        $missing += "Git"
    }
    
    if ($missing.Count -gt 0) {
        Write-Error "Missing prerequisites: $($missing -join ', ')"
        Write-Host @"

Please install the following:
1. Visual Studio 2022 with:
   - Desktop development with C++
   - Windows SDK 10.0.22621.0 or later
   - CMake tools for Windows

2. Python 3.9+ (add to PATH)
3. Ruby 3.0+ (add to PATH)
4. Perl 5.30+ (Strawberry Perl recommended)
5. Git for Windows
"@
        exit 1
    }
    
    Write-Success "All prerequisites found"
}

# ============================================================================
# Environment Setup
# ============================================================================

function Set-BuildEnvironment {
    Write-Status "Setting up build environment..."
    
    # Find Visual Studio
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    $vsPath = & $vsWhere -latest -property installationPath
    
    # Import VS developer environment
    $vcVarsPath = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"
    if (-not (Test-Path $vcVarsPath)) {
        Write-Error "Cannot find vcvars64.bat at: $vcVarsPath"
        exit 1
    }
    
    # Capture environment from vcvars64.bat
    $tempFile = [System.IO.Path]::GetTempFileName()
    cmd /c "`"$vcVarsPath`" && set > `"$tempFile`""
    
    Get-Content $tempFile | ForEach-Object {
        if ($_ -match '^([^=]+)=(.*)$') {
            [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
        }
    }
    
    Remove-Item $tempFile -ErrorAction SilentlyContinue
    
    # Set WebKit-specific variables
    $env:WEBKIT_OUTPUTDIR = $BuildDir
    $env:WEBKIT_LIBRARIES = Join-Path $WebKitRoot "WebKitLibraries\win"
    
    Write-Success "Build environment configured"
}

# ============================================================================
# Build Functions
# ============================================================================

function Invoke-Clean {
    if (-not $Clean) { return }
    
    Write-Status "Cleaning build directory..."
    
    if (Test-Path $BuildDir) {
        Remove-Item $BuildDir -Recurse -Force
    }
    
    Write-Success "Clean complete"
}

function Invoke-CMakeConfigure {
    Write-Status "Configuring CMake..."
    
    $cmakeArgs = @(
        "-S", $WebKitRoot
        "-B", $BuildDir
        "-G", "Ninja"
        "-DCMAKE_BUILD_TYPE=$Config"
        "-DPORT=WinCairo"
        "-DENABLE_WEBKIT=ON"
        "-DENABLE_WEBKIT2=ON"
        "-DENABLE_MINIBROWSER=ON"
        "-DENABLE_TOOLS=ON"
    )
    
    # Disable optional features for faster builds
    $cmakeArgs += @(
        "-DENABLE_WEBDRIVER=OFF"
        "-DENABLE_API_TESTS=OFF"
    )
    
    Write-Host "cmake $($cmakeArgs -join ' ')"
    & cmake @cmakeArgs
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "CMake configuration failed"
        exit 1
    }
    
    Write-Success "CMake configuration complete"
}

function Invoke-Build {
    Write-Status "Building WebKit ($Config)..."
    
    $jobs = $Jobs
    if ($jobs -eq 0) {
        $jobs = [Environment]::ProcessorCount
    }
    
    $buildTargets = switch ($Target) {
        "All" { @() }
        "MiniBrowser" { @("--target", "MiniBrowser") }
        "WebKit2" { @("--target", "WebKit2") }
        "WebCore" { @("--target", "WebCore") }
    }
    
    $buildArgs = @(
        "--build", $BuildDir
        "--config", $Config
        "--parallel", $jobs
    ) + $buildTargets
    
    Write-Host "cmake $($buildArgs -join ' ')"
    & cmake @buildArgs
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed"
        exit 1
    }
    
    Write-Success "Build complete"
}

function Copy-Artifacts {
    Write-Status "Copying artifacts to deps/wincairo..."
    
    $targetDir = Join-Path $PSScriptRoot "..\deps\wincairo"
    $binDir = Join-Path $BuildDir "bin64"
    
    if (-not (Test-Path $targetDir)) {
        New-Item -Path $targetDir -ItemType Directory -Force | Out-Null
    }
    
    # Copy DLLs and EXEs
    $artifacts = @(
        "WebKit2.dll",
        "WebCore.dll",
        "WTF.dll",
        "JavaScriptCore.dll",
        "MiniBrowser.exe"
    )
    
    foreach ($artifact in $artifacts) {
        $src = Join-Path $binDir $artifact
        if (Test-Path $src) {
            Copy-Item $src $targetDir -Force
            Write-Host "  Copied: $artifact"
        }
    }
    
    # Copy PDBs in debug mode
    if ($Config -eq "Debug") {
        Get-ChildItem $binDir -Filter "*.pdb" | ForEach-Object {
            Copy-Item $_.FullName $targetDir -Force
            Write-Host "  Copied: $($_.Name)"
        }
    }
    
    Write-Success "Artifacts copied to deps/wincairo"
}

# ============================================================================
# Main
# ============================================================================

try {
    Write-Host ""
    Write-Host "=" * 60
    Write-Host "  WebKit WinCairo Build Script"
    Write-Host "  Configuration: $Config"
    Write-Host "  Target: $Target"
    Write-Host "  WebKit Root: $WebKitRoot"
    Write-Host "=" * 60
    Write-Host ""
    
    if (-not (Test-Path $WebKitRoot)) {
        Write-Error "WebKit source not found at: $WebKitRoot"
        Write-Host "Please clone WebKit to $WebKitRoot first."
        exit 1
    }
    
    Test-Prerequisites
    Set-BuildEnvironment
    Invoke-Clean
    Invoke-CMakeConfigure
    Invoke-Build
    Copy-Artifacts
    
    Write-Host ""
    Write-Success "WebKit build completed successfully!"
    Write-Host ""
    Write-Host "Build output: $BuildDir"
    Write-Host "Artifacts:    $(Join-Path $PSScriptRoot '..\deps\wincairo')"
    Write-Host ""
    
} catch {
    Write-Error $_.Exception.Message
    exit 1
}

