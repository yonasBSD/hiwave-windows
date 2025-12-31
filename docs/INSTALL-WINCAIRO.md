# Installing WebKit for HiWave (Windows)

HiWave can optionally use WinCairo WebKit instead of WebView2 for a truly Chromium-independent browsing experience.

## Why Use WinCairo WebKit?

- **Chromium Independence**: WebKit is developed by Apple and the open-source community, separate from Google's Chromium project
- **Consistent Rendering**: Same rendering engine across macOS, Linux, and now Windows
- **Privacy-Focused**: WebKit has strong privacy protections built-in
- **Performance**: JavaScriptCore (WebKit's JS engine) offers competitive performance

## Prerequisites

Before building HiWave with WinCairo support, you need to install WebKit on your system.

### Option 1: Download Pre-built Binaries (Recommended)

1. Download the latest WinCairo WebKit build from:
   - Official: https://webkit.org/downloads/
   - Build bots: https://build.webkit.org/

2. Extract to `C:\WebKit` (or your preferred location)

3. Your directory structure should look like:
   ```
   C:\WebKit\
   ├── bin64\
   │   ├── WebKit.dll
   │   ├── JavaScriptCore.dll
   │   ├── libcurl.dll
   │   ├── cairo.dll
   │   └── ... (other DLLs)
   ├── lib64\
   │   └── ... (import libraries)
   └── include\
       └── WebKit\
           └── ... (headers)
   ```

4. Add `C:\WebKit\bin64` to your system PATH:
   - Open System Properties → Advanced → Environment Variables
   - Edit the `Path` variable
   - Add `C:\WebKit\bin64`

### Option 2: Build WebKit from Source

For the latest features or custom builds:

1. Install build dependencies via Chocolatey:
   ```powershell
   choco install -y python311 ruby git cmake gperf llvm ninja strawberryperl
   ```

   > **Note**: Use Python 3.11, not 3.12 (known compatibility issues)

2. Install Visual Studio 2022 with:
   - Desktop development with C++
   - Windows 10 SDK (latest)

3. Clone WebKit:
   ```bash
   git clone https://github.com/WebKit/WebKit.git
   cd WebKit
   ```

4. Build:
   ```bash
   perl Tools\Scripts\build-webkit --release --skip-library-update
   ```

   This takes 30-60 minutes depending on your hardware.

5. Copy the built binaries to `C:\WebKit`

## Building HiWave with WinCairo

Once WebKit is installed:

### Set Environment Variable (if not using default path)

If you installed WebKit somewhere other than `C:\WebKit`:

```powershell
$env:WEBKIT_PATH = "D:\path\to\WebKit"
```

Or set it permanently in System Environment Variables.

### Build HiWave

```bash
# Debug build with WinCairo
cargo build -p hiwave-app --features wincairo

# Release build with WinCairo
cargo build -p hiwave-app --release --features wincairo
```

### Verify the Build

Run HiWave:
```bash
cargo run -p hiwave-app --features wincairo
```

You can verify it's using WebKit by checking the User-Agent string or inspecting the rendering behavior.

## Runtime Requirements

When running HiWave built with WinCairo, the following DLLs must be accessible (in PATH or same directory as hiwave.exe):

- `WebKit.dll` - WebKit core
- `JavaScriptCore.dll` - JavaScript engine
- `libcurl.dll` - Network stack
- `cairo.dll` - Graphics rendering
- `libpng16.dll` - PNG support
- `libjpeg-62.dll` - JPEG support
- `libxml2.dll` - XML parsing
- `libxslt.dll` - XSLT processing
- `icuuc*.dll` - Unicode support
- `icuin*.dll` - ICU internationalization
- `zlib1.dll` - Compression

## Troubleshooting

### "WebKit.dll not found"

Ensure `C:\WebKit\bin64` is in your PATH, or copy the DLLs to the same directory as hiwave.exe.

### Build Fails with "cannot find -lWebKit"

The build script couldn't find the WebKit import libraries. Check that:
1. `WEBKIT_PATH` is set correctly
2. `lib64` directory contains the import libraries

### Crash on Startup

1. Ensure all required DLLs are present
2. Check that you're using 64-bit WebKit (32-bit is not supported)
3. Try running from a terminal to see error messages

### Performance Issues

WinCairo WebKit is still under active development. If you experience issues:
1. Try updating to the latest WebKit build
2. Report issues to the WebKit bug tracker
3. Consider using the WebView2 backend for now (remove `--features wincairo`)

## Switching Between Backends

To switch back to WebView2 (Chromium-based), simply build without the `wincairo` feature:

```bash
cargo build -p hiwave-app --release
```

The default build uses WebView2, which:
- Requires no additional installation (WebView2 is part of Windows)
- Has broader compatibility
- Is more thoroughly tested

## Feature Comparison

| Feature | WebView2 | WinCairo WebKit |
|---------|----------|-----------------|
| Chromium-free | No | Yes |
| Installation | Built-in | Manual |
| Stability | Mature | Improving |
| Memory usage | Higher | Lower |
| JavaScript performance | V8 | JavaScriptCore |
| DevTools | Full Chrome DevTools | Web Inspector |

## References

- [WebKit Windows Port Documentation](https://docs.webkit.org/Ports/WindowsPort.html)
- [WebKit Build Instructions](https://webkit.org/building-webkit/)
- [WinCairo Status Updates](https://webkit.org/status/)
- [WebKit Bug Tracker](https://bugs.webkit.org/)
