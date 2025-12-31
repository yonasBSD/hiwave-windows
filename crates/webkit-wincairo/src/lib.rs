//! Safe Rust wrapper for WinCairo WebKit
//!
//! This crate provides a safe, ergonomic Rust API for embedding WebKit on Windows
//! using the WinCairo port.
//!
//! # Example
//!
//! ```no_run
//! use webkit_wincairo::{WebKitContext, WebKitView, ViewBounds};
//!
//! // Create a shared context
//! let context = WebKitContext::new()?;
//!
//! // Create a view embedded in a window
//! let bounds = ViewBounds::new(0, 0, 800, 600);
//! let view = WebKitView::new(&context, bounds, parent_hwnd)?;
//!
//! // Load a URL
//! view.page().load_url("https://example.com")?;
//! ```

#![cfg(target_os = "windows")]

mod error;
mod context;
mod view;
mod page;
mod callbacks;

pub use error::{WebKitError, Result};
pub use context::WebKitContext;
pub use view::{WebKitView, ViewBounds};
pub use page::WebKitPage;
pub use callbacks::{NavigationDecision, NavigationHandler, TitleChangeHandler, IpcHandler};

// Re-export useful types from sys crate
pub use webkit_wincairo_sys::{
    WKNavigationType,
    WKUserScriptInjectionTime,
    WKUserContentInjectedFrames,
};
