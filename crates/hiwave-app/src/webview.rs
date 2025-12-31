//! WebView abstraction for HiWave
//!
//! This module provides an abstraction layer over different WebView implementations,
//! allowing HiWave to use either WRY (WebView2 on Windows) or WinCairo WebKit.
//!
//! # Architecture
//!
//! The `IWebView` trait defines the common interface that all WebView implementations
//! must provide. This includes operations like:
//! - Loading URLs and HTML content
//! - Evaluating JavaScript
//! - Setting bounds/position
//! - Handling zoom
//!
//! The actual implementation is selected at compile time via feature flags:
//! - Default: Uses WRY (WebView2 on Windows)
//! - `wincairo` feature: Uses WinCairo WebKit

use wry::Rect;
use wry::dpi::{LogicalPosition, LogicalSize};

// Re-export WRY types for convenience
pub use wry::WebView;

#[cfg(not(all(target_os = "windows", feature = "wincairo")))]
pub use wry::WebViewBuilder;

// ============================================================================
// HiWaveWebView - Unified type alias for WebView backends
// ============================================================================

/// Unified WebView type that works with both backends.
///
/// - Default build: Uses `wry::WebView` (WebView2 on Windows)
/// - WinCairo build: Uses `webkit_wincairo::WebKitView`
///
/// Both types implement the `IWebView` trait for common operations.
#[cfg(not(all(target_os = "windows", feature = "wincairo")))]
pub type HiWaveWebView = wry::WebView;

#[cfg(all(target_os = "windows", feature = "wincairo"))]
pub type HiWaveWebView = webkit_wincairo::WebKitView;

// ============================================================================
// Helper functions
// ============================================================================

/// Convert logical position/size to WRY Rect
pub fn make_rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(width, height).into(),
    }
}

/// Helper to safely evaluate a script on a WebView
/// Returns Ok(()) even if the script execution fails (fire-and-forget pattern)
pub fn eval_script(webview: &WebView, script: &str) {
    let _ = webview.evaluate_script(script);
}

/// Helper to safely set bounds on a WebView
pub fn set_webview_bounds(webview: &WebView, x: f64, y: f64, width: f64, height: f64) {
    let _ = webview.set_bounds(make_rect(x, y, width, height));
}

// ============================================================================
// IWebView Trait - Common interface for all WebView implementations
// ============================================================================

/// Common interface for WebView implementations
///
/// This trait abstracts over the differences between WRY (WebView2) and
/// WinCairo WebKit, allowing main.rs to work with either backend.
///
/// Note: This trait is NOT Send because WebView handles are not thread-safe.
/// All WebView operations must be performed on the main/UI thread.
pub trait IWebView {
    /// Load a URL in the WebView
    fn load_url(&self, url: &str);

    /// Load HTML content directly
    fn load_html(&self, html: &str);

    /// Evaluate JavaScript in the WebView (fire-and-forget)
    fn evaluate_script(&self, script: &str);

    /// Set the bounds/position of the WebView
    fn set_bounds(&self, rect: Rect);

    /// Get the current URL (if available)
    fn url(&self) -> Option<String>;

    /// Set zoom level (1.0 = 100%)
    fn set_zoom(&self, level: f64);

    /// Print the current page
    fn print(&self);

    /// Focus the WebView
    fn focus(&self);

    /// Clear all browsing data
    fn clear_all_browsing_data(&self);
}

// ============================================================================
// Arc wrapper implementation - allows Arc<WebView> to be used as IWebView
// ============================================================================

use std::sync::Arc;

impl<T: IWebView> IWebView for Arc<T> {
    fn load_url(&self, url: &str) {
        (**self).load_url(url)
    }

    fn load_html(&self, html: &str) {
        (**self).load_html(html)
    }

    fn evaluate_script(&self, script: &str) {
        (**self).evaluate_script(script)
    }

    fn set_bounds(&self, rect: Rect) {
        (**self).set_bounds(rect)
    }

    fn url(&self) -> Option<String> {
        (**self).url()
    }

    fn set_zoom(&self, level: f64) {
        (**self).set_zoom(level)
    }

    fn print(&self) {
        (**self).print()
    }

    fn focus(&self) {
        (**self).focus()
    }

    fn clear_all_browsing_data(&self) {
        (**self).clear_all_browsing_data()
    }
}

// ============================================================================
// WRY WebView implementation of IWebView
// ============================================================================

impl IWebView for wry::WebView {
    fn load_url(&self, url: &str) {
        let _ = self.load_url(url);
    }

    fn load_html(&self, html: &str) {
        let _ = self.load_html(html);
    }

    fn evaluate_script(&self, script: &str) {
        let _ = wry::WebView::evaluate_script(self, script);
    }

    fn set_bounds(&self, rect: Rect) {
        let _ = wry::WebView::set_bounds(self, rect);
    }

    fn url(&self) -> Option<String> {
        wry::WebView::url(self).ok()
    }

    fn set_zoom(&self, level: f64) {
        let _ = wry::WebView::zoom(self, level);
    }

    fn print(&self) {
        let _ = wry::WebView::print(self);
    }

    fn focus(&self) {
        let _ = wry::WebView::focus(self);
    }

    fn clear_all_browsing_data(&self) {
        let _ = wry::WebView::clear_all_browsing_data(self);
    }
}

// ============================================================================
// Feature-gated WinCairo support
// ============================================================================

#[cfg(all(target_os = "windows", feature = "wincairo"))]
pub mod wincairo_support {
    //! WinCairo WebKit support module
    //!
    //! This module is only compiled when the `wincairo` feature is enabled.
    //! It provides the infrastructure for using WinCairo WebKit instead of WebView2.

    use webkit_wincairo::{WebKitContext, WebKitView, ViewBounds};
    use std::sync::{Arc, OnceLock, Mutex};
    use hiwave_core::HiWaveResult;
    use super::{IWebView, Rect};

    // Re-export types needed by main.rs
    pub use webkit_wincairo::{ViewBounds as WebKitViewBounds, WebKitView as WinCairoWebKitView};

    /// Shared WebKit context for all views
    /// We use OnceLock<Result<...>> to cache both success and failure states
    static WEBKIT_CONTEXT: OnceLock<Result<Arc<WebKitContext>, String>> = OnceLock::new();

    /// Mutex to ensure only one thread attempts initialization
    static INIT_MUTEX: Mutex<()> = Mutex::new(());

    /// Get or create the shared WebKit context
    pub fn get_webkit_context() -> HiWaveResult<Arc<WebKitContext>> {
        // Fast path: check if already initialized
        if let Some(result) = WEBKIT_CONTEXT.get() {
            return result.clone()
                .map_err(|e| hiwave_core::HiWaveError::WebView(e));
        }

        // Slow path: initialize with lock
        let _guard = INIT_MUTEX.lock().unwrap();

        // Double-check after acquiring lock
        if let Some(result) = WEBKIT_CONTEXT.get() {
            return result.clone()
                .map_err(|e| hiwave_core::HiWaveError::WebView(e));
        }

        // Actually initialize - WebKitContext::new() already returns Arc<WebKitContext>
        let result = WebKitContext::new()
            .map_err(|e| e.to_string());

        let _ = WEBKIT_CONTEXT.set(result.clone());

        result.map_err(|e| hiwave_core::HiWaveError::WebView(e))
    }

    /// Create a WinCairo WebView
    ///
    /// # Parameters
    /// - `parent`: Parent window handle (HWND)
    /// - `bounds`: Initial bounds for the view
    ///
    /// # Returns
    /// A new WebKitView instance
    pub fn create_webkit_view(
        parent: windows_sys::Win32::Foundation::HWND,
        bounds: ViewBounds,
    ) -> HiWaveResult<WebKitView> {
        let context = get_webkit_context()?;
        WebKitView::new(&context, bounds, parent)
            .map_err(|e| hiwave_core::HiWaveError::WebView(e.to_string()))
    }

    /// Helper to evaluate script on a WebKit view
    pub fn eval_webkit_script(view: &WebKitView, script: &str) {
        let _ = view.page().evaluate_script_sync(script);
    }

    /// Helper to set bounds on a WebKit view
    pub fn set_webkit_bounds(view: &WebKitView, x: i32, y: i32, width: u32, height: u32) {
        view.set_bounds(ViewBounds::new(x, y, width, height));
    }

    /// Helper to load URL on a WebKit view
    pub fn load_webkit_url(view: &WebKitView, url: &str) -> HiWaveResult<()> {
        view.page().load_url(url)
            .map_err(|e| hiwave_core::HiWaveError::WebView(e.to_string()))
    }

    /// Helper to load HTML on a WebKit view
    pub fn load_webkit_html(view: &WebKitView, html: &str) -> HiWaveResult<()> {
        view.page().load_html(html, None)
            .map_err(|e| hiwave_core::HiWaveError::WebView(e.to_string()))
    }

    // ========================================================================
    // IWebView implementation for WebKitView
    // ========================================================================

    impl IWebView for WebKitView {
        fn load_url(&self, url: &str) {
            let _ = self.page().load_url(url);
        }

        fn load_html(&self, html: &str) {
            let _ = self.page().load_html(html, None);
        }

        fn evaluate_script(&self, script: &str) {
            let _ = self.page().evaluate_script_sync(script);
        }

        fn set_bounds(&self, rect: Rect) {
            // Convert WRY Rect to ViewBounds
            // Note: WRY uses dpi::Position which can be Physical or Logical
            // For simplicity, we treat all as logical coordinates
            let (x, y) = match rect.position {
                wry::dpi::Position::Logical(pos) => (pos.x as i32, pos.y as i32),
                wry::dpi::Position::Physical(pos) => (pos.x, pos.y),
            };
            let (width, height) = match rect.size {
                wry::dpi::Size::Logical(size) => (size.width as u32, size.height as u32),
                wry::dpi::Size::Physical(size) => (size.width, size.height),
            };
            WebKitView::set_bounds(self, ViewBounds::new(x, y, width, height));
        }

        fn url(&self) -> Option<String> {
            self.page().url()
        }

        fn set_zoom(&self, level: f64) {
            self.page().set_zoom(level);
        }

        fn print(&self) {
            // WinCairo WebKit print support - may need implementation
            // For now, this is a no-op
        }

        fn focus(&self) {
            self.set_focus(true);
        }

        fn clear_all_browsing_data(&self) {
            // Clear cookies and cache through the context
            // This would need to be implemented in webkit-wincairo
        }
    }
}

// ============================================================================
// Engine detection
// ============================================================================

/// Check if we're using WinCairo WebKit
#[allow(dead_code)]
pub fn is_wincairo_enabled() -> bool {
    cfg!(all(target_os = "windows", feature = "wincairo"))
}

/// Get the current WebView engine name
pub fn engine_name() -> &'static str {
    #[cfg(all(target_os = "windows", feature = "wincairo"))]
    return "WinCairo WebKit";

    #[cfg(not(all(target_os = "windows", feature = "wincairo")))]
    return "WebView2 (WRY)";
}
