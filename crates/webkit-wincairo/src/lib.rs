//! Safe Rust wrapper for WinCairo WebKit
//!
//! This crate provides a safe, ergonomic Rust API for embedding WebKit on Windows
//! using the WinCairo port.
//!
//! # Example
//!
//! ```no_run
//! use webkit_wincairo::{WebKitContext, WebKitView, ViewBounds};
//! use windows_sys::Win32::Foundation::HWND;
//!
//! fn main() -> webkit_wincairo::Result<()> {
//!     // In a real embedder, pass the parent HWND you are embedding into.
//!     // For documentation purposes we use 0.
//!     let parent_hwnd: HWND = 0;
//!
//!     // Create a shared context
//!     let context = WebKitContext::new()?;
//!
//!     // Create a view embedded in a window
//!     let bounds = ViewBounds::new(0, 0, 800, 600);
//!     let view = WebKitView::new(&context, bounds, parent_hwnd)?;
//!
//!     // Load a URL
//!     view.page().load_url("https://example.com")?;
//!     Ok(())
//! }
//! ```

#![cfg(target_os = "windows")]

mod callbacks;
mod context;
mod error;
mod page;
mod view;

pub use callbacks::{IpcHandler, NavigationDecision, NavigationHandler, TitleChangeHandler};
pub use context::WebKitContext;
pub use error::{Result, WebKitError};
pub use page::WebKitPage;
pub use view::{ViewBounds, WebKitView};

// Re-export useful types from sys crate
pub use webkit_wincairo_sys::{
    wk_string_to_string,
    WKCompletionListenerRef,
    WKNavigationType,
    WKScriptMessageGetBody,
    // Types for script message handler IPC
    WKScriptMessageRef,
    WKStringRef,
    WKUserContentInjectedFrames,
    WKUserScriptInjectionTime,
};
