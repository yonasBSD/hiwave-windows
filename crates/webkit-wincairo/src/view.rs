//! WebKit view management
//!
//! WebKitView is the main interface for displaying web content. Each view
//! represents a browsing session and can be embedded in a Windows HWND.

use std::sync::Arc;
use webkit_wincairo_sys::*;
use windows_sys::Win32::Foundation::{HWND, RECT};
use crate::error::{Result, WebKitError};
use crate::context::WebKitContext;
use crate::page::WebKitPage;

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
        Self { x, y, width, height }
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
/// for rendering web content.
pub struct WebKitView {
    raw: WKViewRef,
    page: WebKitPage,
    #[allow(dead_code)]
    context: Arc<WebKitContext>,
}

impl WebKitView {
    /// Create a new WebKit view
    ///
    /// # Parameters
    ///
    /// - `context`: The WebKit context to use
    /// - `bounds`: Initial position and size
    /// - `parent`: Parent window handle (HWND) to embed in
    ///
    /// # Errors
    ///
    /// Returns an error if the view cannot be created.
    pub fn new(
        context: &Arc<WebKitContext>,
        bounds: ViewBounds,
        parent: HWND,
    ) -> Result<Self> {
        unsafe {
            let rect = bounds.to_rect();
            let raw = WKViewCreate(
                rect,
                context.raw(),
                context.page_group(),
                parent,
            );

            if raw.is_null() {
                return Err(WebKitError::ViewCreationFailed);
            }

            let page_ref = WKViewGetPage(raw);
            if page_ref.is_null() {
                WKRelease(raw);
                return Err(WebKitError::PageCreationFailed);
            }

            let page = WebKitPage::from_raw(page_ref);

            Ok(Self {
                raw,
                page,
                context: Arc::clone(context),
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
    pub fn hwnd(&self) -> HWND {
        unsafe { WKViewGetWindow(self.raw) }
    }

    /// Set the parent window
    pub fn set_parent(&self, parent: HWND) {
        unsafe {
            WKViewSetParentWindow(self.raw, parent);
        }
    }

    /// Set the view bounds
    pub fn set_bounds(&self, bounds: ViewBounds) {
        unsafe {
            WKViewSetBounds(self.raw, bounds.to_rect());
        }
    }

    /// Get the current view bounds
    pub fn bounds(&self) -> ViewBounds {
        unsafe {
            let rect = WKViewGetBounds(self.raw);
            ViewBounds::from_rect(rect)
        }
    }

    /// Set the view position
    pub fn set_position(&self, x: i32, y: i32) {
        unsafe {
            WKViewSetPosition(self.raw, x, y);
        }
    }

    /// Set the view size
    pub fn set_size(&self, width: u32, height: u32) {
        unsafe {
            WKViewSetSize(self.raw, width, height);
        }
    }

    /// Set whether the view draws a transparent background
    pub fn set_transparent_background(&self, transparent: bool) {
        unsafe {
            WKViewSetDrawsTransparentBackground(self.raw, transparent);
        }
    }

    /// Check if the view draws a transparent background
    pub fn has_transparent_background(&self) -> bool {
        unsafe { WKViewDrawsTransparentBackground(self.raw) }
    }

    /// Set the view visibility
    pub fn set_visible(&self, visible: bool) {
        unsafe {
            WKViewSetIsVisible(self.raw, visible);
        }
    }

    /// Check if the view is visible
    pub fn is_visible(&self) -> bool {
        unsafe { WKViewIsVisible(self.raw) }
    }

    /// Set focus to the view
    ///
    /// # Parameters
    ///
    /// - `forward`: If true, focus moves forward through focusable elements
    pub fn set_initial_focus(&self, forward: bool) {
        unsafe {
            WKViewSetInitialFocus(self.raw, forward);
        }
    }

    /// Set whether the view has focus
    pub fn set_focus(&self, focused: bool) {
        unsafe {
            WKViewSetFocus(self.raw, focused);
        }
    }

    /// Check if the view has focus
    pub fn has_focus(&self) -> bool {
        unsafe { WKViewHasFocus(self.raw) }
    }

    /// Show the Web Inspector (developer tools)
    pub fn show_inspector(&self) {
        unsafe {
            WKViewShowInspector(self.raw);
        }
    }

    /// Close the Web Inspector
    pub fn close_inspector(&self) {
        unsafe {
            WKViewCloseInspector(self.raw);
        }
    }

    /// Check if the Web Inspector is visible
    pub fn is_inspector_visible(&self) -> bool {
        unsafe { WKViewIsInspectorVisible(self.raw) }
    }

    /// Scroll the view by a delta
    pub fn scroll_by(&self, delta_x: i32, delta_y: i32) {
        unsafe {
            WKViewScrollBy(self.raw, delta_x, delta_y);
        }
    }
}

impl Drop for WebKitView {
    fn drop(&mut self) {
        unsafe {
            if !self.raw.is_null() {
                WKViewClose(self.raw);
            }
        }
    }
}
