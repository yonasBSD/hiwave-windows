//! WebKit view management
//!
//! WebKitView is the main interface for displaying web content. Each view
//! represents a browsing session and can be embedded in a Windows HWND.
//!
//! Note: WinCairo WebKit provides a minimal WKView API. View positioning,
//! sizing, focus, and visibility are managed through the native HWND
//! returned by `hwnd()` using Windows APIs.

use crate::context::WebKitContext;
use crate::error::{Result, WebKitError};
use crate::page::WebKitPage;
use std::sync::Arc;
use webkit_wincairo_sys::*;
use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{InvalidateRect, UpdateWindow};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetWindowRect, SendMessageW, SetWindowPos, ShowWindow, HWND_TOP, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOW, WM_SIZE,
};

/// Bounds for a WebKit view
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ViewBounds {
    /// Create new view bounds
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Convert to a Windows RECT
    pub fn to_rect(&self) -> RECT {
        RECT {
            left: self.x,
            top: self.y,
            right: self.x + self.width as i32,
            bottom: self.y + self.height as i32,
        }
    }

    /// Create from a Windows RECT
    pub fn from_rect(rect: RECT) -> Self {
        Self {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left).max(0) as u32,
            height: (rect.bottom - rect.top).max(0) as u32,
        }
    }
}

/// A WebKit view that can display web content
///
/// The view is embedded in a parent HWND and manages its own child window
/// for rendering web content. All position/size/visibility changes must
/// be done through the HWND using Windows APIs.
pub struct WebKitView {
    raw: WKViewRef,
    page: WebKitPage,
    #[allow(dead_code)]
    context: Arc<WebKitContext>,
    activated: std::cell::Cell<bool>,
}

impl WebKitView {
    /// Create a new WebKit view
    ///
    /// # Parameters
    ///
    /// - `context`: The WebKit context to use
    /// - `bounds`: Initial position and size (used for initial positioning via HWND)
    /// - `parent`: Parent window handle (HWND) to embed in
    ///
    /// # Errors
    ///
    /// Returns an error if the view cannot be created.
    pub fn new(context: &Arc<WebKitContext>, bounds: ViewBounds, parent: HWND) -> Result<Self> {
        unsafe {
            log::debug!("Creating WebKit view...");

            // Create page configuration
            let page_config = WKPageConfigurationCreate();
            if page_config.is_null() {
                log::error!("WKPageConfigurationCreate returned NULL");
                return Err(WebKitError::ViewCreationFailed);
            }

            // Set context on configuration
            WKPageConfigurationSetContext(page_config, context.raw());

            // Set page group if available
            let page_group = context.page_group();
            if !page_group.is_null() {
                WKPageConfigurationSetPageGroup(page_config, page_group);
            }

            // Create and set user content controller (required for user scripts and message handlers)
            let user_content_controller = WKUserContentControllerCreate();
            if !user_content_controller.is_null() {
                WKPageConfigurationSetUserContentController(page_config, user_content_controller);
                WKRelease(user_content_controller); // Config retains it
            } else {
                log::warn!("Failed to create user content controller - scripts may not work!");
            }

            // Create preferences and enable JavaScript + accelerated compositing
            let prefs = WKPreferencesCreate();
            if !prefs.is_null() {
                WKPreferencesSetJavaScriptEnabled(prefs, true);
                WKPreferencesSetDeveloperExtrasEnabled(prefs, true);
                // Enable accelerated compositing for rendering
                WKPreferencesSetAcceleratedCompositingEnabled(prefs, true);
                WKPageConfigurationSetPreferences(page_config, prefs);
                WKRelease(prefs); // Config retains it
            } else {
                log::warn!("Failed to create preferences - JavaScript may be disabled!");
            }

            // Create the view
            let rect = bounds.to_rect();
            let raw = WKViewCreate(rect, page_config, parent);

            // Release configuration (view retains what it needs)
            WKRelease(page_config);

            if raw.is_null() {
                log::error!("WKViewCreate returned NULL");
                return Err(WebKitError::ViewCreationFailed);
            }

            let page_ref = WKViewGetPage(raw);
            if page_ref.is_null() {
                WKRelease(raw);
                return Err(WebKitError::PageCreationFailed);
            }

            // Set custom backing scale factor for DPI awareness
            let dpi_scale = {
                let hdc = windows_sys::Win32::Graphics::Gdi::GetDC(parent);
                let dpi = windows_sys::Win32::Graphics::Gdi::GetDeviceCaps(hdc, 88); // LOGPIXELSX = 88
                windows_sys::Win32::Graphics::Gdi::ReleaseDC(parent, hdc);
                (dpi as f64 / 96.0).max(1.0)
            };
            WKPageSetCustomBackingScaleFactor(page_ref, dpi_scale);

            let page = WebKitPage::from_raw(page_ref);

            // Set initial bounds
            let hwnd = WKViewGetWindow(raw);
            if hwnd != 0 {
                SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    bounds.x,
                    bounds.y,
                    bounds.width as i32,
                    bounds.height as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }

            log::debug!("WebKit view created successfully");

            Ok(Self {
                raw,
                page,
                context: Arc::clone(context),
                activated: std::cell::Cell::new(false),
            })
        }
    }

    /// Get the page associated with this view
    pub fn page(&self) -> &WebKitPage {
        &self.page
    }

    /// Get a mutable reference to the page
    pub fn page_mut(&mut self) -> &mut WebKitPage {
        &mut self.page
    }

    /// Get the native window handle (HWND) for this view
    ///
    /// Use this HWND with Windows APIs to control the view's
    /// position, size, visibility, and focus.
    pub fn hwnd(&self) -> HWND {
        unsafe { WKViewGetWindow(self.raw) }
    }

    /// Get the raw WKViewRef (for advanced use)
    pub fn raw(&self) -> WKViewRef {
        self.raw
    }

    /// Set the parent window
    pub fn set_parent(&self, parent: HWND) {
        unsafe {
            WKViewSetParentWindow(self.raw, parent);
            WKViewWindowAncestryDidChange(self.raw);
        }
    }

    /// Set the view bounds using Windows API
    ///
    /// **Known Limitation:** WinCairo WebKit's accelerated compositing layer
    /// does not properly respond to HWND size changes. The window will be
    /// resized at the OS level, but the rendered content may not update.
    /// See docs/WINCAIRO-LIMITATIONS.md for details.
    pub fn set_bounds(&self, bounds: ViewBounds) {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            unsafe {
                SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    bounds.x,
                    bounds.y,
                    bounds.width as i32,
                    bounds.height as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );

                // Send WM_SIZE to notify WebKit of the new size
                let lparam = ((bounds.height as LPARAM) << 16) | (bounds.width as LPARAM & 0xFFFF);
                SendMessageW(hwnd, WM_SIZE, 0 as WPARAM, lparam);

                // Notify WebKit that the window changed
                WKViewWindowAncestryDidChange(self.raw);

                // Force a repaint
                InvalidateRect(hwnd, std::ptr::null(), 1);

                // Try to force WebKit to repaint
                let page_ref = WKViewGetPage(self.raw);
                if !page_ref.is_null() {
                    WKPageForceRepaint(page_ref, std::ptr::null_mut(), force_repaint_callback);
                }
            }
        }
    }

    /// Get the current view bounds
    pub fn bounds(&self) -> ViewBounds {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            unsafe {
                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                GetWindowRect(hwnd, &mut rect);
                ViewBounds::from_rect(rect)
            }
        } else {
            ViewBounds::default()
        }
    }

    /// Get the client area size
    pub fn client_size(&self) -> (u32, u32) {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            unsafe {
                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                GetClientRect(hwnd, &mut rect);
                (
                    (rect.right - rect.left).max(0) as u32,
                    (rect.bottom - rect.top).max(0) as u32,
                )
            }
        } else {
            (0, 0)
        }
    }

    /// Set the view position
    pub fn set_position(&self, x: i32, y: i32) {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            let (width, height) = self.client_size();
            unsafe {
                SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    x,
                    y,
                    width as i32,
                    height as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }

    /// Set the view size
    pub fn set_size(&self, width: u32, height: u32) {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            let bounds = self.bounds();
            unsafe {
                SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    bounds.x,
                    bounds.y,
                    width as i32,
                    height as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }

    /// Set the view visibility
    pub fn set_visible(&self, visible: bool) {
        let hwnd = self.hwnd();

        // When showing for the first time, activate WebKit rendering
        if visible && !self.activated.get() {
            log::debug!("Activating WebKit view for first time");
            unsafe {
                // First show the window so WebKit has a surface to render to
                if hwnd != 0 {
                    ShowWindow(hwnd, SW_SHOW);
                }

                // Tell WebKit it's in a window
                WKViewSetIsInWindow(self.raw, true);
                WKViewWindowAncestryDidChange(self.raw);

                // Force redraw
                if hwnd != 0 {
                    InvalidateRect(hwnd, std::ptr::null(), 1);
                    UpdateWindow(hwnd);
                }
            }
            self.activated.set(true);
            return;
        }

        if hwnd != 0 {
            unsafe {
                ShowWindow(hwnd, if visible { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    /// Bring the view to the front of the z-order
    pub fn bring_to_front(&self) {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            unsafe {
                SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }

    /// Notify that view is in/out of a window
    pub fn set_is_in_window(&self, is_in_window: bool) {
        unsafe {
            WKViewSetIsInWindow(self.raw, is_in_window);
        }
    }

    /// Set whether to use offscreen rendering
    pub fn set_uses_offscreen_rendering(&self, uses_offscreen: bool) {
        unsafe {
            WKViewSetUsesOffscreenRendering(self.raw, uses_offscreen);
        }
    }

    /// Set scroll offset for next resize
    pub fn set_scroll_offset_on_resize(&self, x: f64, y: f64) {
        unsafe {
            WKViewSetScrollOffsetOnNextResize(self.raw, x, y);
        }
    }

    /// Set focus to the view using Windows API
    pub fn focus(&self) {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            unsafe {
                SetFocus(hwnd);
            }
        }
    }

    /// Show the Web Inspector (developer tools)
    pub fn show_inspector(&self) {
        // Use the page's inspector if available
        unsafe {
            let inspector = WKPageGetInspector(self.page.raw());
            if !inspector.is_null() {
                // Inspector show is done through the page
                WKPageSetAllowsRemoteInspection(self.page.raw(), true);
            }
        }
    }
}

impl Drop for WebKitView {
    fn drop(&mut self) {
        unsafe {
            if !self.raw.is_null() {
                // Notify view is leaving window
                WKViewSetIsInWindow(self.raw, false);
                // Release the view
                WKRelease(self.raw);
            }
        }
    }
}

/// Callback for WKPageForceRepaint - no-op, fire and forget
extern "C" fn force_repaint_callback(_context: *mut std::ffi::c_void, _error: WKTypeRef) {}
