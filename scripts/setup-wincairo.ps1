# Setup script for WinCairo WebKit DLLs
# This script downloads and installs WebKit WinCairo binaries for HiWave
#
# Usage:
#   .\scripts\setup-wincairo.ps1
#   .\scripts\setup-wincairo.ps1 -WebKitZip "path\to\webkit.zip"

param(
    [string]$WebKitZip = "",
    [string]$WebKitRequirementsZip = "",
    [switch]$Force,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Configuration
$DepsDir = Join-Path $PSScriptRoot "..\deps\wincairo"
$TempDir = Join-Path $env:TEMP "hiwave-webkit-setup"

# Required DLLs - Core WebKit
# Note: WinCairo uses WebKit2.dll, not WebKit.dll
$CoreDlls = @(
    "WebKit2.dll",
    "JavaScriptCore.dll",
    "WebCore.dll",
    "WTF.dll",
    "WebKitLegacy.dll"
)

# Required DLLs - Dependencies (from WebKitRequirements)
$DependencyDlls = @(
    "cairo.dll",
    "libcurl.dll",
    "libpng16.dll",
    "zlib1.dll",
    "libjpeg-9.dll",
    "libxml2.dll",
    "libxslt.dll",
    "icuuc*.dll",
    "icuin*.dll",
    "icudt*.dll",
    "sqlite3.dll",
    "libwebp.dll",
    "pthreadVC3.dll",
    "brotlicommon.dll",
    "brotlidec.dll",
    "nghttp2.dll",
    "libssh2.dll",
    "libeay32.dll",
    "ssleay32.dll"
)

function Show-Help {
    Write-Host @"
WinCairo WebKit Setup Script for HiWave
========================================

This script downloads and configures WebKit WinCairo DLLs for building HiWave
with the 'wincairo' feature.

USAGE:
    .\setup-wincairo.ps1 [OPTIONS]

OPTIONS:
    -WebKitZip <path>            Path to pre-downloaded WebKit zip file
    -WebKitRequirementsZip <path> Path to pre-downloaded WebKitRequirements zip
    -Force                       Overwrite existing DLLs
    -Help                        Show this help message

MANUAL DOWNLOAD:
    If automatic download fails, manually download:

    1. WebKit WinCairo Build:
       - Go to: https://build.webkit.org/#/builders/1192
       - Click a green (successful) build number
       - Download the "Archive" from "compile-webkit" step

    2. WebKitRequirements:
       - Go to: https://github.com/WebKitForWindows/WebKitRequirements/releases
       - Download the latest release zip

    Then run:
       .\setup-wincairo.ps1 -WebKitZip "path\to\webkit.zip" -WebKitRequirementsZip "path\to\requirements.zip"

OUTPUT:
    DLLs are installed to: deps\wincairo\

"@
}

function Test-DllsExist {
    if (-not (Test-Path $DepsDir)) {
        return $false
    }

    # Note: WinCairo uses WebKit2.dll, not WebKit.dll
    $webkitDll = Join-Path $DepsDir "WebKit2.dll"
    $jscDll = Join-Path $DepsDir "JavaScriptCore.dll"

    return (Test-Path $webkitDll) -and (Test-Path $jscDll)
}

function New-TempDirectory {
    if (Test-Path $TempDir) {
        Remove-Item -Recurse -Force $TempDir
    }
    New-Item -ItemType Directory -Path $TempDir | Out-Null
}

function Get-WebKitFromBuildbot {
    Write-Host "Attempting to download WebKit from buildbot..." -ForegroundColor Cyan

    # The buildbot API can be used to find recent builds
    # For now, we'll provide instructions for manual download
    Write-Host ""
    Write-Host "Automatic download from WebKit buildbot is not yet implemented." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Please download manually:" -ForegroundColor White
    Write-Host "  1. Go to: https://build.webkit.org/#/builders/1192" -ForegroundColor Gray
    Write-Host "  2. Click a GREEN (successful) build number" -ForegroundColor Gray
    Write-Host "  3. Click 'Archive' link under 'compile-webkit'" -ForegroundColor Gray
    Write-Host "  4. Re-run: .\setup-wincairo.ps1 -WebKitZip 'path\to\downloaded.zip'" -ForegroundColor Gray
    Write-Host ""

    return $null
}

function Get-WebKitRequirements {
    Write-Host "Downloading WebKitRequirements..." -ForegroundColor Cyan

    # Try to get the latest release from GitHub
    try {
        $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/WebKitForWindows/WebKitRequirements/releases/latest"
        $asset = $releases.assets | Where-Object { $_.name -like "*64*" -or $_.name -like "*x64*" } | Select-Object -First 1

        if ($null -eq $asset) {
            $asset = $releases.assets | Select-Object -First 1
        }

        if ($null -ne $asset) {
            $downloadUrl = $asset.browser_download_url
            $downloadPath = Join-Path $TempDir "WebKitRequirements.zip"

            Write-Host "  Downloading from: $downloadUrl" -ForegroundColor Gray
            Invoke-WebRequest -Uri $downloadUrl -OutFile $downloadPath

            return $downloadPath
        }
    }
    catch {
        Write-Host "  Failed to download WebKitRequirements: $_" -ForegroundColor Yellow
    }

    return $null
}

function Expand-WebKitArchive {
    param([string]$ZipPath, [string]$DestDir)

    Write-Host "Extracting $ZipPath..." -ForegroundColor Cyan

    $extractDir = Join-Path $TempDir "extract"
    Expand-Archive -Path $ZipPath -DestinationPath $extractDir -Force

    # Find DLLs in the extracted content
    $dllFiles = Get-ChildItem -Path $extractDir -Recurse -Include "*.dll"

    foreach ($dll in $dllFiles) {
        $destPath = Join-Path $DestDir $dll.Name
        if (-not (Test-Path $destPath) -or $Force) {
            Copy-Item -Path $dll.FullName -Destination $destPath -Force
            Write-Host "  Copied: $($dll.Name)" -ForegroundColor Gray
        }
    }

    # Also copy import libraries (.lib files)
    $libFiles = Get-ChildItem -Path $extractDir -Recurse -Include "*.lib"
    foreach ($lib in $libFiles) {
        $destPath = Join-Path $DestDir $lib.Name
        if (-not (Test-Path $destPath) -or $Force) {
            Copy-Item -Path $lib.FullName -Destination $destPath -Force
            Write-Host "  Copied: $($lib.Name)" -ForegroundColor Gray
        }
    }
}

function Copy-FromExistingWebKit {
    # Check if WebKit is installed at the default location
    $defaultPath = "C:\WebKit"

    if (Test-Path "$defaultPath\bin64\WebKit.dll") {
        Write-Host "Found existing WebKit installation at $defaultPath" -ForegroundColor Green

        $response = Read-Host "Copy DLLs from $defaultPath? (Y/n)"
        if ($response -eq "" -or $response -match "^[Yy]") {
            # Copy from bin64
            $sourceDir = "$defaultPath\bin64"
            $dllFiles = Get-ChildItem -Path $sourceDir -Include "*.dll" -File

            foreach ($dll in $dllFiles) {
                $destPath = Join-Path $DepsDir $dll.Name
                Copy-Item -Path $dll.FullName -Destination $destPath -Force
                Write-Host "  Copied: $($dll.Name)" -ForegroundColor Gray
            }

            # Copy import libraries from lib64
            if (Test-Path "$defaultPath\lib64") {
                $libFiles = Get-ChildItem -Path "$defaultPath\lib64" -Include "*.lib" -File
                foreach ($lib in $libFiles) {
                    $destPath = Join-Path $DepsDir $lib.Name
                    Copy-Item -Path $lib.FullName -Destination $destPath -Force
                    Write-Host "  Copied: $($lib.Name)" -ForegroundColor Gray
                }
            }

            return $true
        }
    }

    return $false
}

function Show-Summary {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "WinCairo WebKit Setup Complete!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "DLLs installed to: $DepsDir" -ForegroundColor White
    Write-Host ""
    Write-Host "To build HiWave with WinCairo WebKit:" -ForegroundColor Cyan
    Write-Host "  cargo build -p hiwave-app --features wincairo" -ForegroundColor White
    Write-Host ""

    # List installed DLLs
    $dlls = Get-ChildItem -Path $DepsDir -Include "*.dll" -File
    Write-Host "Installed DLLs ($($dlls.Count)):" -ForegroundColor Cyan
    foreach ($dll in $dlls | Sort-Object Name) {
        $size = [math]::Round($dll.Length / 1MB, 2)
        Write-Host "  $($dll.Name) ($size MB)" -ForegroundColor Gray
    }
}

# Main script
if ($Help) {
    Show-Help
    exit 0
}

Write-Host ""
Write-Host "WinCairo WebKit Setup for HiWave" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Create deps directory
if (-not (Test-Path $DepsDir)) {
    New-Item -ItemType Directory -Path $DepsDir | Out-Null
    Write-Host "Created: $DepsDir" -ForegroundColor Gray
}

# Check if already set up
if ((Test-DllsExist) -and -not $Force) {
    Write-Host "WebKit DLLs already present in $DepsDir" -ForegroundColor Green
    Write-Host "Use -Force to overwrite." -ForegroundColor Gray
    exit 0
}

# Create temp directory
New-TempDirectory

try {
    # Option 1: Use provided zip files
    if ($WebKitZip -and (Test-Path $WebKitZip)) {
        Expand-WebKitArchive -ZipPath $WebKitZip -DestDir $DepsDir
    }
    else {
        # Option 2: Copy from existing installation
        if (-not (Copy-FromExistingWebKit)) {
            # Option 3: Try to download
            $downloaded = Get-WebKitFromBuildbot
            if ($null -eq $downloaded) {
                Write-Host ""
                Write-Host "WebKit DLLs not available. Please download manually." -ForegroundColor Yellow
                Write-Host "See: docs/INSTALL-WINCAIRO.md for instructions" -ForegroundColor Gray
            }
        }
    }

    # Handle WebKitRequirements
    if ($WebKitRequirementsZip -and (Test-Path $WebKitRequirementsZip)) {
        Expand-WebKitArchive -ZipPath $WebKitRequirementsZip -DestDir $DepsDir
    }
    else {
        $reqZip = Get-WebKitRequirements
        if ($null -ne $reqZip) {
            Expand-WebKitArchive -ZipPath $reqZip -DestDir $DepsDir
        }
    }

    # Show summary if DLLs were installed
    if (Test-DllsExist) {
        Show-Summary
    }
}
finally {
    # Cleanup
    if (Test-Path $TempDir) {
        Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
    }
}
