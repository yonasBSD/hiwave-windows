# Native-Win32 Mode Implementation Guide

## Overview

This guide walks through making `--features native-win32` work, which enables 100% RustKit rendering (no WebView2/Tao dependencies).

**Current Status:**
- ✅ `main_win32.rs` module exists and is complete
- ❌ `main.rs` has unconditional imports of wry/tao/muda
- ❌ Build fails when these crates aren't available

**Goal:**
Make all wry/tao/muda usage conditional so native-win32 mode compiles without them.

---

## Step 1: Make Cargo.toml Dependencies Optional

**File:** `crates/hiwave-app/Cargo.toml`

This is already done ✅ (you have it from earlier). The key lines are:

```toml
[features]
default = ["native-win32"]  # Changed to native-win32
native-win32 = ["rustkit-engine", "rustkit-viewhost", "rustkit-bindings", "rustkit-net", "windows", "tokio"]

[dependencies]
wry = { workspace = true, optional = true }
tao = { workspace = true, optional = true }
muda = { workspace = true, optional = true }
```

---

## Step 2: Fix main.rs Imports (Lines 1-100)

**File:** `crates/hiwave-app/src/main.rs`

**Current code (lines ~15-30):**
```rust
use muda::Menu;
use platform::get_platform_manager;
#[cfg(target_os = "macos")]
use platform::menu_ids;
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Icon, WindowBuilder},
};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use webview::{engine_name, HiWaveWebView, IWebView};
use wry::{Rect, WebViewBuilder};
```

**Replace with:**
```rust
// Imports only needed for hybrid/WebView mode
#[cfg(not(feature = "native-win32"))]
use muda::Menu;
#[cfg(not(feature = "native-win32"))]
use platform::get_platform_manager;
#[cfg(all(target_os = "macos", not(feature = "native-win32")))]
use platform::menu_ids;
#[cfg(not(feature = "native-win32"))]
use std::sync::{atomic::AtomicBool, Arc, Mutex};
#[cfg(not(feature = "native-win32"))]
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Icon, WindowBuilder},
};
#[cfg(not(feature = "native-win32"))]
use webview::{engine_name, HiWaveWebView, IWebView};
#[cfg(not(feature = "native-win32"))]
use wry::{Rect, WebViewBuilder};

// Always need logging
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;
```

---

## Step 3: Fix Constants (Lines ~30-45)

**Current code:**
```rust
const CHROME_HEIGHT_DEFAULT: u32 = 104;
const CHROME_HEIGHT_SMALL: u32 = 148;
const CHROME_HEIGHT_EXPANDED: u32 = 460;
const SHELF_HEIGHT_DEFAULT: u32 = 0;
const SHELF_HEIGHT_EXPANDED: u32 = 280;
const SIDEBAR_WIDTH: f64 = 220.0;
```

**Replace with:**
```rust
#[cfg(not(feature = "native-win32"))]
const CHROME_HEIGHT_DEFAULT: u32 = 104;
#[cfg(not(feature = "native-win32"))]
const CHROME_HEIGHT_SMALL: u32 = 148;
#[cfg(not(feature = "native-win32"))]
const CHROME_HEIGHT_EXPANDED: u32 = 460;
#[cfg(not(feature = "native-win32"))]
const SHELF_HEIGHT_DEFAULT: u32 = 0;
#[cfg(not(feature = "native-win32"))]
const SHELF_HEIGHT_EXPANDED: u32 = 280;
#[cfg(not(feature = "native-win32"))]
const SIDEBAR_WIDTH: f64 = 220.0;
```

---

## Step 4: Fix Module Declarations (Lines ~45-55)

**Current code:**
```rust
mod ipc;
mod state;
mod webview;

#[cfg(all(target_os = "windows", feature = "rustkit"))]
mod webview_rustkit;

#[cfg(all(target_os = "windows", feature = "rustkit"))]
mod shield_adapter;

#[cfg(all(target_os = "windows", feature = "native-win32"))]
mod main_win32;
```

**Replace with:**
```rust
// Hybrid mode modules
#[cfg(not(feature = "native-win32"))]
mod ipc;
#[cfg(not(feature = "native-win32"))]
mod state;
#[cfg(not(feature = "native-win32"))]
mod webview;

// RustKit modules for hybrid mode
#[cfg(all(target_os = "windows", feature = "rustkit", not(feature = "native-win32")))]
mod webview_rustkit;

#[cfg(all(target_os = "windows", feature = "rustkit", not(feature = "native-win32")))]
mod shield_adapter;

// Native Win32 module
#[cfg(all(target_os = "windows", feature = "native-win32"))]
mod main_win32;
```

---

## Step 5: Fix Module Imports (Lines ~55-60)

**Current code:**
```rust
use hiwave_shield::ResourceType;
use ipc::{IpcMessage, JS_BRIDGE};
use state::AppState;
```

**Replace with:**
```rust
#[cfg(not(feature = "native-win32"))]
use hiwave_shield::ResourceType;
#[cfg(not(feature = "native-win32"))]
use ipc::{IpcMessage, JS_BRIDGE};
#[cfg(not(feature = "native-win32"))]
use state::AppState;
```

---

## Step 6: Fix HTML Constants (Lines ~60-80)

**Current code:**
```rust
const CHROME_HTML: &str = include_str!("ui/chrome.html");
const SHELF_HTML: &str = include_str!("ui/shelf.html");
const NEW_TAB_URL: &str = "hiwave://newtab";
const ABOUT_HTML: &str = include_str!("ui/about.html");
const ABOUT_URL: &str = "hiwave://about";
const SETTINGS_HTML: &str = include_str!("ui/settings.html");
const REPORT_HTML: &str = include_str!("ui/report.html");
const REPORT_URL: &str = "hiwave://report";
const FIND_IN_PAGE_HELPER: &str = include_str!("ui/find_in_page.js");
const CONTEXT_MENU_HELPER: &str = include_str!("ui/context_menu.js");
const AUDIO_DETECTOR: &str = include_str!("ui/audio_detector.js");
const AUTOFILL_HELPER: &str = include_str!("ui/autofill.js");
const CHART_JS: &str = include_str!("ui/chart.umd.min.js");
```

**Replace with:**
```rust
#[cfg(not(feature = "native-win32"))]
const CHROME_HTML: &str = include_str!("ui/chrome.html");
#[cfg(not(feature = "native-win32"))]
const SHELF_HTML: &str = include_str!("ui/shelf.html");
#[cfg(not(feature = "native-win32"))]
const NEW_TAB_URL: &str = "hiwave://newtab";
#[cfg(not(feature = "native-win32"))]
const ABOUT_HTML: &str = include_str!("ui/about.html");
#[cfg(not(feature = "native-win32"))]
const ABOUT_URL: &str = "hiwave://about";
#[cfg(not(feature = "native-win32"))]
const SETTINGS_HTML: &str = include_str!("ui/settings.html");
#[cfg(not(feature = "native-win32"))]
const REPORT_HTML: &str = include_str!("ui/report.html");
#[cfg(not(feature = "native-win32"))]
const REPORT_URL: &str = "hiwave://report";
#[cfg(not(feature = "native-win32"))]
const FIND_IN_PAGE_HELPER: &str = include_str!("ui/find_in_page.js");
#[cfg(not(feature = "native-win32"))]
const CONTEXT_MENU_HELPER: &str = include_str!("ui/context_menu.js");
#[cfg(not(feature = "native-win32"))]
const AUDIO_DETECTOR: &str = include_str!("ui/audio_detector.js");
#[cfg(not(feature = "native-win32"))]
const AUTOFILL_HELPER: &str = include_str!("ui/autofill.js");
#[cfg(not(feature = "native-win32"))]
const CHART_JS: &str = include_str!("ui/chart.umd.min.js");
```

---

## Step 7: Fix create_window_icon() Function (Lines ~80-150)

**Current code:**
```rust
fn create_window_icon() -> Option<Icon> {
    const SIZE: u32 = 32;
    // ... rest of function
}
```

**Replace with:**
```rust
#[cfg(not(feature = "native-win32"))]
fn create_window_icon() -> Option<Icon> {
    const SIZE: u32 = 32;
    // ... rest of function (leave unchanged)
}
```

---

## Step 8: Fix ShelfScope Enum (Lines ~150-170)

**Current code:**
```rust
#[derive(Debug, Clone, Copy)]
enum ShelfScope {
    Workspace,
    All,
}

impl ShelfScope {
    fn as_str(self) -> &'static str {
        match self {
            ShelfScope::Workspace => "workspace",
            ShelfScope::All => "all",
        }
    }
}
```

**Replace with:**
```rust
#[cfg(not(feature = "native-win32"))]
#[derive(Debug, Clone, Copy)]
enum ShelfScope {
    Workspace,
    All,
}

#[cfg(not(feature = "native-win32"))]
impl ShelfScope {
    fn as_str(self) -> &'static str {
        match self {
            ShelfScope::Workspace => "workspace",
            ShelfScope::All => "all",
        }
    }
}
```

---

## Step 9: Find All Remaining Functions/Types

Now you need to find ALL other functions, enums, structs, and impl blocks that use wry/tao types.

**Search command:**
```powershell
Select-String -Path "P:\petes_code\ClaudeCode\hiwave-windows\crates\hiwave-app\src\main.rs" -Pattern "^(fn |struct |enum |impl |type )" | Select-Object LineNumber, Line
```

For EACH function/struct/enum that is NOT inside the `#[cfg(feature = "native-win32")]` block (lines 595-603), add:
```rust
#[cfg(not(feature = "native-win32"))]
```

**EXCEPT:**
- Keep `fn main()` unconditional (it handles both modes)
- The native-win32 block itself (lines 595-603) stays as-is

---

## Step 10: Automated Approach (Faster)

Instead of manually finding every function, you can wrap the ENTIRE hybrid mode in one big cfg block:

**Find line 605** (right after the native-win32 early return)

**Add:**
```rust
    // Native Win32 mode returns above, everything below is hybrid mode
    #[cfg(not(feature = "native-win32"))]
    {
```

**Find the LAST line of main()** (probably around line 2700+)

**Add closing brace:**
```rust
    } // end cfg(not(feature = "native-win32"))
```

This wraps the entire hybrid mode in one conditional block.

**BUT** you still need to fix the imports/constants at the top (Steps 2-8).

---

## Step 11: Test the Build

```powershell
cd P:\petes_code\ClaudeCode\hiwave-windows

# Clean build
cargo clean

# Test native-win32 build
cargo build --release --no-default-features --features native-win32
```

**Expected result:** Should compile with NO errors

**If you get errors:** Look at the error messages - they'll tell you which items still need `#[cfg(not(feature = "native-win32"))]`

---

## Step 12: Test Runtime

Once it compiles:

```powershell
# Run the binary
.\target\release\hiwave.exe
```

**What to check:**
1. Does it launch without crashing?
2. Do you see "WebView engine: RustKit (native-win32)" in logs?
3. Can you type a URL in the address bar?
4. Navigate to wikipedia.org - does it load?
5. Navigate to twitter.com - does it work?
6. Navigate to youtube.com - does video play?

---

## Step 13: Debug Issues

**If it crashes on launch:**
- Check Windows Event Viewer for crash details
- Look for error logs in console output
- Check `main_win32.rs` - maybe there's a bug there

**If websites don't load:**
- Open DevTools (if available) to check console errors
- Try simpler sites first (example.com, google.com)
- Check if it's a networking issue vs rendering issue

**If it's slow/unresponsive:**
- That's expected for alpha - not a blocker
- Focus on "does it work" not "is it fast"

---

## Step 14: Compare to Hybrid Mode

To verify native-win32 is actually different:

```powershell
# Build hybrid mode
cargo build --release --no-default-features --features rustkit

# Run it
.\target\release\hiwave.exe
# Should say "WebView engine: RustKit" (not "native-win32")
```

Check dependency tree:
```powershell
# Native-win32 - should NOT include wry/tao
cargo tree -p hiwave-app --no-default-features --features native-win32 | Select-String "wry|tao"
# Should return NOTHING

# Hybrid mode - SHOULD include wry/tao  
cargo tree -p hiwave-app --no-default-features --features rustkit | Select-String "wry|tao"
# Should show wry and tao
```

---

## Estimated Time Breakdown

- **Steps 1-8** (fix imports/constants): 30-45 minutes
- **Step 9** (find remaining items): 15-30 minutes  
- **Step 10** (wrap main body): 5-10 minutes
- **Step 11** (fix compilation errors): 15-60 minutes (depends on how many you missed)
- **Step 12-13** (test and debug): 30-120 minutes (depends on bugs)
- **Step 14** (verify): 10 minutes

**Total: 2-4.5 hours**

---

## Shortcuts to Save Time

### Option A: Minimal Fix (Faster)
Just fix Steps 2-8, then wrap the entire main() body in `#[cfg(not(feature = "native-win32"))]` as described in Step 10. This is faster but less clean.

### Option B: Comprehensive Fix (Cleaner)
Do all steps including manually finding each function. Takes longer but results in cleaner code.

**Recommendation:** Use Option A (minimal fix) to get it working, then refactor later if needed.

---

## Success Criteria

✅ `cargo build --no-default-features --features native-win32` compiles with 0 errors
✅ `cargo tree` shows NO wry/tao dependencies
✅ Binary launches and shows "WebView engine: RustKit (native-win32)"
✅ Can navigate to and render wikipedia.org
✅ Can navigate to and render twitter.com
✅ JavaScript works (Twitter requires it)

Once you hit all these criteria, you can launch with the claim:
> "HiWave runs on 100% RustKit - zero WebView2, zero Chromium, zero WebKit dependencies. Everything from browser chrome to web content is custom Rust code."

---

## If You Get Stuck

**Common issues:**

1. **"Cannot find Icon in scope"** → You forgot to conditionally import `tao::window::Icon`

2. **"Cannot find WebView"** → Some function is still using wry::WebView - find it and add cfg

3. **"Cannot find AppState"** → Some code is using AppState outside the cfg block

4. **Build succeeds but crashes** → Check main_win32.rs for bugs, add more logging

5. **Websites don't render** → RustKit engine issue, not a native-win32 issue - that's a separate problem

---

## Alternative: Launch Hybrid First

If you hit issues and it's taking too long, you can:

1. Change default back to `rustkit` instead of `native-win32`
2. Launch hybrid mode tomorrow
3. Fix native-win32 properly over the next week
4. Ship native-win32 as v0.2.0

This is the pragmatic path that gets you launched faster while still having an impressive achievement.

**Your call!**

---

## Final Checklist Before Launch

Once native-win32 works:

- [ ] Test on fresh Windows install (VM if possible)
- [ ] Record demo video showing it render Wikipedia/Twitter/YouTube
- [ ] Update README to say "100% RustKit, zero WebView2"
- [ ] Run `.\package-windows.ps1` to create release ZIP
- [ ] Upload to GitHub Releases
- [ ] Post to Hacker News
- [ ] Post to r/rust
- [ ] Quote tweet the original inspiration

**Good luck! This is the home stretch.**
