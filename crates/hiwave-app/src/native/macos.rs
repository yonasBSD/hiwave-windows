//! Native macOS Entry Point for HiWave
//!
//! This module provides a pure Cocoa entry point for HiWave that uses RustKit
//! for all rendering, completely bypassing wry/tao.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  Cocoa Main Window (NSWindow)           │
//! ├─────────────────────────────────────────┤
//! │  RustKit Chrome View (tabs, toolbar)    │
//! ├─────────────────────────────────────────┤
//! │  RustKit Content View (web pages)       │
//! ├─────────────────────────────────────────┤
//! │  RustKit Shelf View (command palette)   │
//! └─────────────────────────────────────────┘
//! ```

#![cfg(target_os = "macos")]

use tracing::{error, info};

/// Entry point for native macOS mode.
///
/// This function is called from main.rs when native mode is enabled on macOS.
pub fn run_native() -> Result<(), String> {
    info!("Starting HiWave in native macOS mode");

    // TODO: Implement macOS native browser
    // This requires:
    // 1. rustkit_viewhost::macos::MacOSViewHost
    // 2. rustkit_text macOS backend (Core Text)
    // 3. rustkit_a11y macOS backend (NSAccessibility)

    error!("macOS native mode not yet implemented");
    Err("macOS native mode not yet implemented - use hybrid mode".into())
}

