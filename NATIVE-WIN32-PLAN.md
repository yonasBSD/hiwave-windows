# Native-Win32 Mode - Implementation Plan

## Current Status
- ✅ `main_win32.rs` exists with full native implementation
- ❌ `main.rs` has unconditional wry/tao imports
- ❌ Build fails when wry/tao are not available

## Fix Required

### 1. Make imports conditional in main.rs

Change:
```rust
use muda::Menu;
use tao::{...};
use wry::{Rect, WebViewBuilder};
```

To:
```rust
#[cfg(not(feature = "native-win32"))]
use muda::Menu;
#[cfg(not(feature = "native-win32"))]
use tao::{...};
#[cfg(not(feature = "native-win32"))]
use wry::{Rect, WebViewBuilder};
```

### 2. Conditional main() function

The main() function needs to either:
- Run hybrid mode (wry/tao + RustKit content)
- Run native-win32 mode (100% RustKit)

### 3. Testing checklist

Once it compiles:
- [ ] Does it launch?
- [ ] Can you navigate to Wikipedia?
- [ ] Does Wikipedia render?
- [ ] Does JavaScript work?
- [ ] Does Twitter load?
- [ ] Does YouTube play?

## Estimated Time: 2-4 hours

This is a significant refactor because main.rs is ~2500 lines and heavily uses wry/tao types.

## Decision Point

**Quick fix (30 min):** Keep hybrid as default, add note that native-win32 is experimental
**Full fix (2-4 hours):** Make native-win32 work completely

**What do you want to do?**

I can:
A) Help you do the full refactor (will take time, I'll guide you)
B) Recommend launching hybrid mode with "native-win32 coming in v0.2"
C) Create a separate binary for native-win32 mode (keeps main.rs simple)

Your choice?
