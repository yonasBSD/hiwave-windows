# HiWave Installation Guide

This guide will walk you through installing HiWave from source. Whether you're a Rust veteran or have never touched a command line, follow these steps and you'll have HiWave running in no time.

---

## Table of Contents

1. [Prerequisites by Platform](#prerequisites-by-platform)
   - [macOS](#macos)
   - [Windows](#windows)
   - [Linux (Ubuntu/Debian)](#linux-ubuntudebian)
   - [Linux (Fedora/RHEL)](#linux-fedorarhel)
   - [Linux (Arch)](#linux-arch)
2. [Installing Rust](#installing-rust)
3. [Cloning HiWave](#cloning-hiwave)
4. [Building HiWave](#building-hiwave)
5. [Running HiWave](#running-hiwave)
6. [Troubleshooting](#troubleshooting)
7. [Updating HiWave](#updating-hiwave)
8. [Uninstalling](#uninstalling)

---

## Prerequisites by Platform

Before installing Rust or building HiWave, you need some system-level tools and libraries.

### macOS

#### 1. Install Xcode Command Line Tools

Open Terminal (Applications → Utilities → Terminal) and run:

```bash
xcode-select --install
```

A popup will appear asking to install the tools. Click "Install" and wait for it to complete (usually 5-10 minutes).

**How to verify:**
```bash
xcode-select -p
# Should output: /Library/Developer/CommandLineTools
```

#### 2. Install Homebrew (Recommended)

Homebrew is a package manager for macOS. While not strictly required, it makes installing additional tools easier.

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the on-screen instructions. After installation, you may need to add Homebrew to your PATH:

```bash
# For Apple Silicon Macs (M1/M2/M3):
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"

# For Intel Macs:
echo 'eval "$(/usr/local/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/usr/local/bin/brew shellenv)"
```

**How to verify:**
```bash
brew --version
# Should output: Homebrew 4.x.x
```

#### 3. No Additional Dependencies Needed

macOS includes WebKit, which HiWave uses for rendering. You're ready to install Rust!

---

### Windows

#### 1. Install Visual Studio Build Tools

Rust on Windows requires the Microsoft C++ build tools.

**Option A: Full Visual Studio (if you want an IDE)**
1. Download [Visual Studio Community](https://visualstudio.microsoft.com/vs/community/)
2. Run the installer
3. Select "Desktop development with C++"
4. Make sure these components are checked:
   - MSVC v143 - VS 2022 C++ x64/x86 build tools
   - Windows 10/11 SDK
5. Click Install (requires ~8GB)

**Option B: Build Tools Only (smaller, recommended)**
1. Download [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Run `vs_BuildTools.exe`
3. Select "Desktop development with C++"
4. Install (requires ~6GB)

**How to verify:**
Open a new Command Prompt or PowerShell and run:
```powershell
cl
# Should show Microsoft C/C++ Compiler version info
```

If `cl` is not found, you may need to run from "Developer Command Prompt for VS 2022" instead.

#### 2. Install WebView2 Runtime (Usually Pre-installed)

Windows 10 (version 1803+) and Windows 11 come with WebView2. If you're on an older version:

1. Download from [Microsoft WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
2. Run the Evergreen Bootstrapper


#### 3. Install Git for Windows

1. Download from [git-scm.com](https://git-scm.com/download/win)
2. Run the installer with default options
3. Restart your terminal

**How to verify:**
```powershell
git --version
# Should output: git version 2.x.x
```

---

### Linux (Ubuntu/Debian)

#### 1. Update Package Lists

```bash
sudo apt update
```

#### 2. Install Build Essentials

```bash
sudo apt install -y build-essential
```

This installs GCC, G++, make, and other compilation tools.

#### 3. Install Required Libraries

HiWave needs GTK3 and WebKitGTK for rendering:

```bash
# For Ubuntu 22.04+ / Debian 12+
sudo apt install -y \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev

# For Ubuntu 20.04 / Debian 11 (older WebKitGTK version)
sudo apt install -y \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libappindicator3-dev \
    librsvg2-dev
```

#### 4. Install Git

```bash
sudo apt install -y git
```

**How to verify everything:**
```bash
gcc --version        # Should show GCC version
pkg-config --version # Should show pkg-config version
git --version        # Should show git version
```

---

### Linux (Fedora/RHEL)

#### 1. Install Development Tools

```bash
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y gcc-c++
```

#### 2. Install Required Libraries

```bash
sudo dnf install -y \
    openssl-devel \
    gtk3-devel \
    webkit2gtk4.1-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

#### 3. Install Git

```bash
sudo dnf install -y git
```

---

### Linux (Arch)

#### 1. Install Base Development

```bash
sudo pacman -Syu --needed base-devel
```

#### 2. Install Required Libraries

```bash
sudo pacman -S --needed \
    openssl \
    gtk3 \
    webkit2gtk-4.1 \
    libappindicator-gtk3 \
    librsvg
```

#### 3. Install Git

```bash
sudo pacman -S --needed git
```

---

## Installing Rust

Rust is installed using `rustup`, the official Rust toolchain installer.

### All Platforms

#### 1. Download and Run Rustup

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows (PowerShell):**
```powershell
# Download and run rustup-init.exe from:
# https://win.rustup.rs/x86_64

# Or use winget:
winget install Rustlang.Rustup
```

#### 2. Follow the Prompts

When prompted, choose option `1` for default installation:

```
1) Proceed with standard installation (default - just press enter)
2) Customize installation
3) Cancel installation
```

Press Enter to proceed.

#### 3. Configure Your Shell

**macOS / Linux:**
```bash
# Add Cargo (Rust's package manager) to your PATH
source "$HOME/.cargo/env"

# Or restart your terminal
```

**Windows:**
Close and reopen your terminal (Command Prompt or PowerShell).

#### 4. Verify Installation

```bash
rustc --version
# Should output: rustc 1.75.0 (or newer)

cargo --version
# Should output: cargo 1.75.0 (or newer)

rustup --version
# Should output: rustup 1.x.x
```

If any command fails, try restarting your terminal or computer.

---

## Cloning HiWave

Now let's get the HiWave source code.

#### 1. Choose a Location

Pick where you want to store the code. Common choices:

**macOS / Linux:**
```bash
cd ~/Projects  # or ~/code, ~/dev, etc.
# If the folder doesn't exist:
mkdir -p ~/Projects && cd ~/Projects
```

**Windows:**
```powershell
cd C:\Users\YourName\Projects  # or wherever you prefer
# If the folder doesn't exist:
mkdir C:\Users\YourName\Projects
cd C:\Users\YourName\Projects
```

#### 2. Clone the Repository

```bash
git clone https://github.com/petecopeland/hiwave.git
cd hiwave
```

If the repository is private:
```bash
git clone https://github.com/petecopeland/hiwave.git
# Enter your GitHub username and personal access token when prompted
```

#### 3. Verify the Clone

```bash
ls  # macOS/Linux
dir # Windows

# You should see:
# Cargo.toml
# crates/
# Planning/
# README.md
# etc.
```

---

## Building HiWave

Now for the exciting part - building the browser!

#### 1. Build in Debug Mode (Faster, for Development)

```bash
cargo build -p hiwave-app
```

**What to expect:**
- First build downloads dependencies (~200 packages)
- Takes 2-10 minutes depending on your computer
- You'll see lots of "Compiling..." messages
- When complete, you'll see "Finished dev [unoptimized + debuginfo]"

#### 2. Build in Release Mode (Slower, but Optimized)

```bash
cargo build --release -p hiwave-app
```

**What to expect:**
- Takes 5-15 minutes
- Produces a smaller, faster binary
- When complete, you'll see "Finished release [optimized]"

#### 3. Where's the Binary?

| Build Mode | Location |
|------------|----------|
| Debug | `target/debug/hiwave-app` (+ `.exe` on Windows) |
| Release | `target/release/hiwave-app` (+ `.exe` on Windows) |

---

## Running HiWave

#### Option 1: Run via Cargo (Recommended for Development)

```bash
# Debug mode (faster builds, slower runtime)
cargo run -p hiwave-app

# Release mode (slower builds, faster runtime)
cargo run --release -p hiwave-app
```

#### Option 2: Run the Binary Directly

**macOS / Linux:**
```bash
# Debug
./target/debug/hiwave-app

# Release
./target/release/hiwave-app
```

**Windows:**
```powershell
# Debug
.\target\debug\hiwave-app.exe

# Release
.\target\release\hiwave-app.exe
```

#### What You Should See

A window should open with:
- Teal/cyan themed interface
- Address bar at the top
- Sidebar with workspaces
- A "New Tab" landing page

If the window opens, congratulations - HiWave is running!

---

## Troubleshooting

### Common Issues

#### "error: linker `cc` not found" (Linux)
You're missing build tools:
```bash
sudo apt install build-essential  # Ubuntu/Debian
sudo dnf groupinstall "Development Tools"  # Fedora
```

#### "error: failed to run custom build command for `openssl-sys`"
You need OpenSSL development files:
```bash
sudo apt install libssl-dev  # Ubuntu/Debian
sudo dnf install openssl-devel  # Fedora
brew install openssl  # macOS (if using non-system OpenSSL)
```

#### "error: could not find native static library `webkit2gtk-4.1`" (Linux)
Install WebKitGTK:
```bash
sudo apt install libwebkit2gtk-4.1-dev  # Ubuntu 22.04+
sudo apt install libwebkit2gtk-4.0-dev  # Ubuntu 20.04
```

#### "LINK : fatal error LNK1181" (Windows)
Visual Studio Build Tools aren't installed correctly:
1. Run Visual Studio Installer
2. Modify your installation
3. Ensure "Desktop development with C++" is checked

#### "Permission denied" when running
Make the binary executable:
```bash
chmod +x ./target/release/hiwave-app
```

#### Build takes forever / runs out of memory
Try limiting parallel jobs:
```bash
cargo build -p hiwave-app -j 2  # Use only 2 CPU cores
```

#### HiWave window is blank (white screen)
This usually indicates a WebView initialization issue:
- **Linux:** Make sure WebKitGTK is installed correctly
- **Windows:** Ensure WebView2 Runtime is installed
- **macOS:** This shouldn't happen - file an issue

#### "rustc: error: no such command" 
Rust isn't in your PATH:
```bash
source "$HOME/.cargo/env"  # macOS/Linux
# Or restart your terminal
```

### Getting Help

If you're still stuck:

1. **Check existing issues:** [github.com/petecopeland/hiwave/issues](https://github.com/petecopeland/hiwave/issues)
2. **File a new issue** with:
   - Your operating system and version
   - The exact error message
   - Output of `rustc --version` and `cargo --version`
3. **Join the community:** [Discord/Matrix link when available]

---

## Updating HiWave

When new versions are released:

```bash
# Navigate to your hiwave directory
cd ~/Projects/hiwave  # or wherever you cloned it

# Pull the latest changes
git pull origin main

# Rebuild
cargo build --release -p hiwave-app

# Run
cargo run --release -p hiwave-app
```

### Updating Rust

Keep Rust up to date for best compatibility:

```bash
rustup update
```

---

## Uninstalling

### Remove HiWave

Simply delete the cloned folder:

```bash
rm -rf ~/Projects/hiwave  # macOS/Linux
# or
rmdir /s /q C:\Users\YourName\Projects\hiwave  # Windows
```

HiWave stores its data in:
- **macOS:** `~/Library/Application Support/hiwave/`
- **Linux:** `~/.local/share/hiwave/`
- **Windows:** `%LOCALAPPDATA%\hiwave\`

Delete these folders to remove all saved data (workspaces, vault, history).

### Remove Rust (Optional)

If you want to uninstall Rust entirely:

```bash
rustup self uninstall
```

---

## Quick Reference

### Minimum System Requirements

| Platform | Requirement |
|----------|-------------|
| macOS | 12.0 (Monterey) or later |
| Windows | 10 (version 1803) or later with WebView2 |
| Linux | GTK 3.24+, WebKitGTK 4.1+ |
| RAM | 4GB minimum, 8GB recommended |
| Disk | ~2GB for toolchain + build artifacts |

### Build Commands Cheat Sheet

```bash
# Check if code compiles (fast, no binary)
cargo check -p hiwave-app

# Build debug version
cargo build -p hiwave-app

# Build release version
cargo build --release -p hiwave-app

# Build and run
cargo run -p hiwave-app
cargo run --release -p hiwave-app

# Run tests
cargo test --workspace

# Clean build artifacts (if things get weird)
cargo clean
```

### File Locations

| What | Where |
|------|-------|
| Source code | `~/Projects/hiwave/` (or wherever you cloned) |
| Debug binary | `target/debug/hiwave-app` |
| Release binary | `target/release/hiwave-app` |
| User data (macOS) | `~/Library/Application Support/hiwave/` |
| User data (Linux) | `~/.local/share/hiwave/` |
| User data (Windows) | `%LOCALAPPDATA%\hiwave\` |

---

*Having trouble? File an issue at [github.com/petecopeland/hiwave/issues](https://github.com/petecopeland/hiwave/issues)*
