//! WebKit view functions (Windows-specific)
//!
//! WKView is the Windows-specific view class that embeds WebKit in an HWND.
//! This is the primary way to display web content on Windows.

use super::wk_types::*;
use windows_sys::Win32::Foundation::{HWND, RECT};

extern "C" {
    // ========== View Creation ==========

    /// Create a new WKView
    ///
    /// # Parameters
    /// - `rect`: Initial bounds for the view
    /// - `context`: The WebKit context
    /// - `page_group`: The page group
    /// - `parent`: Parent HWND to embed in (can be null for standalone window)
    pub fn WKViewCreate(
        rect: RECT,
        context: WKContextRef,
        page_group: WKPageGroupRef,
        parent: HWND,
    ) -> WKViewRef;

    /// Create a WKView with a specific configuration
    pub fn WKViewCreateWithConfiguration(
        rect: RECT,
        configuration: WKPageConfigurationRef,
        parent: HWND,
    ) -> WKViewRef;

    // ========== View Properties ==========

    /// Get the page associated with this view
    pub fn WKViewGetPage(view: WKViewRef) -> WKPageRef;

    /// Get the native window handle (HWND)
    pub fn WKViewGetWindow(view: WKViewRef) -> HWND;

    /// Set the parent window
    pub fn WKViewSetParentWindow(view: WKViewRef, parent: HWND);

    /// Get the parent window
    pub fn WKViewGetParentWindow(view: WKViewRef) -> HWND;

    // ========== View Size/Position ==========

    /// Set the size of the view
    pub fn WKViewSetSize(view: WKViewRef, width: u32, height: u32);

    /// Set the bounds of the view
    pub fn WKViewSetBounds(view: WKViewRef, bounds: RECT);

    /// Get the bounds of the view
    pub fn WKViewGetBounds(view: WKViewRef) -> RECT;

    /// Set the position of the view
    pub fn WKViewSetPosition(view: WKViewRef, x: i32, y: i32);

    // ========== View Display ==========

    /// Set whether the view draws a transparent background
    pub fn WKViewSetDrawsTransparentBackground(
        view: WKViewRef,
        draws_transparent: bool,
    );

    /// Check if the view draws a transparent background
    pub fn WKViewDrawsTransparentBackground(view: WKViewRef) -> bool;

    /// Set whether the view is visible
    pub fn WKViewSetIsVisible(view: WKViewRef, visible: bool);

    /// Check if the view is visible
    pub fn WKViewIsVisible(view: WKViewRef) -> bool;

    // ========== Focus ==========

    /// Set the initial focus (called when the view is first shown)
    pub fn WKViewSetInitialFocus(view: WKViewRef, forward: bool);

    /// Set whether the view should be focused
    pub fn WKViewSetFocus(view: WKViewRef, focused: bool);

    /// Check if the view has focus
    pub fn WKViewHasFocus(view: WKViewRef) -> bool;

    // ========== Cursor ==========

    /// Set the cursor for the view
    pub fn WKViewSetCursor(view: WKViewRef, cursor: HWND);

    // ========== Scrolling ==========

    /// Scroll the view by a delta
    pub fn WKViewScrollBy(view: WKViewRef, delta_x: i32, delta_y: i32);

    // ========== Developer Tools ==========

    /// Show the Web Inspector
    pub fn WKViewShowInspector(view: WKViewRef);

    /// Close the Web Inspector
    pub fn WKViewCloseInspector(view: WKViewRef);

    /// Check if the Web Inspector is visible
    pub fn WKViewIsInspectorVisible(view: WKViewRef) -> bool;

    // ========== Lifecycle ==========

    /// Close the view (release resources)
    pub fn WKViewClose(view: WKViewRef);
}

/// Helper struct for working with view bounds
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ViewBounds {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn to_rect(&self) -> RECT {
        RECT {
            left: self.x,
            top: self.y,
            right: self.x + self.width as i32,
            bottom: self.y + self.height as i32,
        }
    }

    pub fn from_rect(rect: RECT) -> Self {
        Self {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left) as u32,
            height: (rect.bottom - rect.top) as u32,
        }
    }
}
