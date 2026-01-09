# README Update - Run Modes Section

Replace lines 111-166 (the "Run Modes" section) with:

```markdown
### Run Modes

HiWave supports multiple rendering modes on Windows:

| Mode | Command | Description |
|------|---------|-------------|
| **RustKit Hybrid** (default) | `cargo run --release` or `.\scripts\run-rustkit.ps1` | RustKit for content, WebView2 for chrome UI |
| **WebView2 Fallback** | `.\scripts\run-webview2.ps1` | System WebView2 for all rendering |
| **Native Win32** (experimental) | `.\scripts\run-native-win32.ps1` | 100% RustKit (work in progress) |

#### RustKit Hybrid Mode (Default) ⭐
```powershell
# Using convenience script
.\scripts\run-rustkit.ps1

# Or directly with cargo
cargo run --release -p hiwave-app
cargo run -p hiwave-app --no-default-features --features rustkit
```

Hybrid mode uses **RustKit for all web content**, WebView2 only for browser chrome:
- ✅ **All websites rendered by RustKit** - Wikipedia, Twitter, YouTube, etc.
- ✅ Engine-level ad/tracker blocking via Brave's adblock-rust
- ✅ Memory-safe Rust rendering pipeline
- 🎨 Browser chrome (tabs, address bar) uses WebView2 for stability
- ⚡ Best balance of performance and compatibility

**This is what the demo video shows!**

#### WebView2 Fallback Mode
```powershell
.\scripts\run-webview2.ps1

# Or directly with cargo
cargo run -p hiwave-app --no-default-features --features webview-fallback
```

WebView2 fallback uses Microsoft Edge WebView2 for all rendering:
- ✅ Maximum web compatibility
- 🔍 Useful for debugging RustKit-specific issues
- 🌐 Full Chromium rendering support

#### Native Win32 Mode (Experimental)
```powershell
.\scripts\run-native-win32.ps1

# Or directly with cargo
cargo run -p hiwave-app --no-default-features --features native-win32
```

**Status:** Work in progress. This mode aims to use RustKit for everything (chrome UI + content):
- 🚧 Currently being developed
- 🚀 Will use 100% Rust rendering when complete
- 🔧 No wry/tao/WebView2 dependencies
- ⚡ Fastest startup and lowest memory when done

**Want to help?** See [NATIVE-WIN32-IMPLEMENTATION.md](NATIVE-WIN32-IMPLEMENTATION.md) for details.
```

## Why This Change:

1. **Honesty:** Makes it clear hybrid is default (matches what works)
2. **Accurate:** Native-win32 is marked experimental (doesn't compile yet)
3. **No overselling:** Clear about what the demo shows
4. **Still impressive:** RustKit rendering all web content is the achievement

Save this and update your README.
