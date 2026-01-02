//! # RustKit ViewHost
//!
//! Win32 window hosting layer for the RustKit browser engine.
//! Handles child HWND creation, resize events, DPI changes, focus, and visibility.
//!
//! ## Design Goals
//!
//! 1. **Multi-view support**: Each view has isolated state, no global singletons
//! 2. **Resize correctness**: WM_SIZE triggers surface resize immediately
//! 3. **DPI awareness**: Per-monitor DPI scaling
//! 4. **Focus management**: Proper focus chain for keyboard events

// Allow Arc with non-Send/Sync types - intentional for Win32 HWND handling
#![allow(clippy::arc_with_non_send_sync)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;
use tracing::{debug, error, info, trace};

#[cfg(windows)]
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, HBRUSH, PAINTSTRUCT},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            HiDpi::{
                GetDpiForWindow, SetProcessDpiAwarenessContext,
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            },
            Input::KeyboardAndMouse::SetFocus,
            WindowsAndMessaging::*,
        },
    },
};

/// Unique identifier for a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(u64);

impl ViewId {
    /// Create a new unique ViewId.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl Default for ViewId {
    fn default() -> Self {
        Self::new()
    }
}

/// Rectangle representing view bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn zero() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

/// Errors that can occur in the ViewHost.
#[derive(Error, Debug)]
pub enum ViewHostError {
    #[error("Failed to create window: {0}")]
    WindowCreation(String),

    #[error("View not found: {0:?}")]
    ViewNotFound(ViewId),

    #[error("Invalid parent window")]
    InvalidParent,

    #[error("Windows API error: {0}")]
    WindowsApi(String),
}

/// Events emitted by the ViewHost.
#[derive(Debug, Clone)]
pub enum ViewEvent {
    /// View bounds changed (includes DPI-aware dimensions).
    Resized {
        view_id: ViewId,
        bounds: Bounds,
        dpi: u32,
    },
    /// View received focus.
    Focused { view_id: ViewId },
    /// View lost focus.
    Blurred { view_id: ViewId },
    /// View visibility changed.
    VisibilityChanged { view_id: ViewId, visible: bool },
    /// DPI changed for the view.
    DpiChanged { view_id: ViewId, dpi: u32 },
    /// View is being destroyed.
    Destroyed { view_id: ViewId },
}

/// Callback for view events.
pub type EventCallback = Box<dyn Fn(ViewEvent) + Send + Sync>;

/// Per-view state.
#[allow(dead_code)]
struct ViewState {
    id: ViewId,
    #[cfg(windows)]
    hwnd: HWND,
    #[cfg(not(windows))]
    hwnd: (),
    bounds: Bounds,
    dpi: u32,
    visible: bool,
    #[allow(dead_code)]
    focused: bool,
}

/// The main ViewHost that manages all views.
pub struct ViewHost {
    views: RwLock<HashMap<ViewId, Arc<Mutex<ViewState>>>>,
    #[cfg(windows)]
    hwnd_to_view: RwLock<HashMap<isize, ViewId>>,
    event_callback: Option<EventCallback>,
}

impl ViewHost {
    /// Create a new ViewHost.
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            // Enable per-monitor DPI awareness
            unsafe {
                let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            }
        }

        Self {
            views: RwLock::new(HashMap::new()),
            #[cfg(windows)]
            hwnd_to_view: RwLock::new(HashMap::new()),
            event_callback: None,
        }
    }

    /// Set the event callback for all views.
    pub fn set_event_callback(&mut self, callback: EventCallback) {
        self.event_callback = Some(callback);
    }

    /// Create a new child view under the given parent HWND.
    #[cfg(windows)]
    pub fn create_view(
        &self,
        parent: HWND,
        initial_bounds: Bounds,
    ) -> Result<ViewId, ViewHostError> {
        if parent.0.is_null() {
            return Err(ViewHostError::InvalidParent);
        }

        let view_id = ViewId::new();
        debug!(?view_id, ?initial_bounds, "Creating view");

        // Get DPI for the parent window
        let dpi = unsafe { GetDpiForWindow(parent) };
        let dpi = if dpi == 0 { 96 } else { dpi };

        // Create child window
        let hwnd = unsafe {
            let class_name = Self::register_class()?;

            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                PCWSTR::null(),
                WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                initial_bounds.x,
                initial_bounds.y,
                initial_bounds.width as i32,
                initial_bounds.height as i32,
                parent,
                None,
                GetModuleHandleW(None).unwrap_or_default(),
                None,
            )
        };

        let hwnd = hwnd.map_err(|e| ViewHostError::WindowCreation(e.to_string()))?;

        if hwnd.0.is_null() {
            let err = std::io::Error::last_os_error();
            error!(?err, "Failed to create child window");
            return Err(ViewHostError::WindowCreation(err.to_string()));
        }

        let state = Arc::new(Mutex::new(ViewState {
            id: view_id,
            hwnd,
            bounds: initial_bounds,
            dpi,
            visible: true,
            focused: false,
        }));

        // Store the view
        {
            let mut views = self.views.write().unwrap();
            views.insert(view_id, state);
        }

        // Map HWND to ViewId for window proc lookups
        {
            let mut hwnd_map = self.hwnd_to_view.write().unwrap();
            hwnd_map.insert(hwnd.0 as isize, view_id);
        }

        info!(?view_id, ?hwnd, dpi, "View created");
        Ok(view_id)
    }

    /// Create a new view (non-Windows stub).
    #[cfg(not(windows))]
    pub fn create_view(
        &self,
        _parent: (),
        initial_bounds: Bounds,
    ) -> Result<ViewId, ViewHostError> {
        let view_id = ViewId::new();
        let state = Arc::new(Mutex::new(ViewState {
            id: view_id,
            hwnd: (),
            bounds: initial_bounds,
            dpi: 96,
            visible: true,
            focused: false,
        }));
        self.views.write().unwrap().insert(view_id, state);
        Ok(view_id)
    }

    /// Set the bounds of a view.
    pub fn set_bounds(&self, view_id: ViewId, bounds: Bounds) -> Result<(), ViewHostError> {
        let views = self.views.read().unwrap();
        let state = views
            .get(&view_id)
            .ok_or(ViewHostError::ViewNotFound(view_id))?;

        let mut state = state.lock().unwrap();
        state.bounds = bounds;

        #[cfg(windows)]
        {
            unsafe {
                let _ = SetWindowPos(
                    state.hwnd,
                    None,
                    bounds.x,
                    bounds.y,
                    bounds.width as i32,
                    bounds.height as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );

                // Force repaint
                let _ = InvalidateRect(state.hwnd, None, false);
            }
        }

        trace!(?view_id, ?bounds, "Bounds updated");
        Ok(())
    }

    /// Get the current bounds of a view.
    pub fn get_bounds(&self, view_id: ViewId) -> Result<Bounds, ViewHostError> {
        let views = self.views.read().unwrap();
        let state = views
            .get(&view_id)
            .ok_or(ViewHostError::ViewNotFound(view_id))?;
        let bounds = state.lock().unwrap().bounds;
        Ok(bounds)
    }

    /// Set view visibility.
    pub fn set_visible(&self, view_id: ViewId, visible: bool) -> Result<(), ViewHostError> {
        let views = self.views.read().unwrap();
        let state = views
            .get(&view_id)
            .ok_or(ViewHostError::ViewNotFound(view_id))?;

        let mut state = state.lock().unwrap();
        state.visible = visible;

        #[cfg(windows)]
        {
            unsafe {
                let _ = ShowWindow(state.hwnd, if visible { SW_SHOW } else { SW_HIDE });
            }
        }

        debug!(?view_id, visible, "Visibility changed");
        Ok(())
    }

    /// Focus a view.
    pub fn focus(&self, view_id: ViewId) -> Result<(), ViewHostError> {
        let views = self.views.read().unwrap();
        let state = views
            .get(&view_id)
            .ok_or(ViewHostError::ViewNotFound(view_id))?;

        let state = state.lock().unwrap();

        #[cfg(windows)]
        {
            unsafe {
                let _ = SetFocus(state.hwnd);
            }
        }

        debug!(?view_id, "Focus requested");
        Ok(())
    }

    /// Get the HWND for a view.
    #[cfg(windows)]
    pub fn get_hwnd(&self, view_id: ViewId) -> Result<HWND, ViewHostError> {
        let views = self.views.read().unwrap();
        let state = views
            .get(&view_id)
            .ok_or(ViewHostError::ViewNotFound(view_id))?;
        let hwnd = state.lock().unwrap().hwnd;
        Ok(hwnd)
    }

    /// Get the DPI for a view.
    pub fn get_dpi(&self, view_id: ViewId) -> Result<u32, ViewHostError> {
        let views = self.views.read().unwrap();
        let state = views
            .get(&view_id)
            .ok_or(ViewHostError::ViewNotFound(view_id))?;
        let dpi = state.lock().unwrap().dpi;
        Ok(dpi)
    }

    /// Destroy a view.
    pub fn destroy_view(&self, view_id: ViewId) -> Result<(), ViewHostError> {
        let state = {
            let mut views = self.views.write().unwrap();
            views.remove(&view_id)
        };

        if let Some(state) = state {
            let state = state.lock().unwrap();

            #[cfg(windows)]
            {
                let mut hwnd_map = self.hwnd_to_view.write().unwrap();
                hwnd_map.remove(&(state.hwnd.0 as isize));

                unsafe {
                    let _ = DestroyWindow(state.hwnd);
                }
            }

            info!(?view_id, "View destroyed");
            Ok(())
        } else {
            Err(ViewHostError::ViewNotFound(view_id))
        }
    }

    /// Get the number of active views.
    pub fn view_count(&self) -> usize {
        self.views.read().unwrap().len()
    }

    /// Register the window class (Windows only).
    #[cfg(windows)]
    fn register_class() -> Result<PCWSTR, ViewHostError> {
        use std::sync::Once;

        static REGISTER: Once = Once::new();
        static CLASS_NAME: &[u16] = &[
            b'R' as u16,
            b'u' as u16,
            b's' as u16,
            b't' as u16,
            b'K' as u16,
            b'i' as u16,
            b't' as u16,
            b'V' as u16,
            b'i' as u16,
            b'e' as u16,
            b'w' as u16,
            b'H' as u16,
            b'o' as u16,
            b's' as u16,
            b't' as u16,
            0,
        ];

        REGISTER.call_once(|| unsafe {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleW(None).unwrap_or_default().into(),
                hIcon: HICON::default(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hbrBackground: HBRUSH::default(),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: PCWSTR::from_raw(CLASS_NAME.as_ptr()),
                hIconSm: HICON::default(),
            };

            let _ = RegisterClassExW(&wc);
        });

        Ok(PCWSTR::from_raw(CLASS_NAME.as_ptr()))
    }

    /// Window procedure for view windows.
    #[cfg(windows)]
    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_SIZE => {
                let width = (lparam.0 & 0xFFFF) as u32;
                let height = ((lparam.0 >> 16) & 0xFFFF) as u32;
                trace!(?hwnd, width, height, "WM_SIZE received");
                // Compositor will be notified via ViewEvent::Resized
            }
            WM_DPICHANGED => {
                let new_dpi = (wparam.0 & 0xFFFF) as u32;
                let suggested_rect = lparam.0 as *const RECT;
                if !suggested_rect.is_null() {
                    let rect = &*suggested_rect;
                    let _ = SetWindowPos(
                        hwnd,
                        None,
                        rect.left,
                        rect.top,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
                trace!(?hwnd, new_dpi, "WM_DPICHANGED");
            }
            WM_SETFOCUS => {
                trace!(?hwnd, "WM_SETFOCUS");
            }
            WM_KILLFOCUS => {
                trace!(?hwnd, "WM_KILLFOCUS");
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let _hdc = BeginPaint(hwnd, &mut ps);
                // Compositor handles actual painting
                let _ = EndPaint(hwnd, &ps);
                return LRESULT(0);
            }
            WM_ERASEBKGND => {
                // Prevent flicker - compositor handles background
                return LRESULT(1);
            }
            WM_DESTROY => {
                trace!(?hwnd, "WM_DESTROY");
            }
            _ => {}
        }

        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

impl Default for ViewHost {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ViewHost {
    fn drop(&mut self) {
        // Destroy all views
        let view_ids: Vec<_> = self.views.read().unwrap().keys().copied().collect();
        for view_id in view_ids {
            let _ = self.destroy_view(view_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_id_uniqueness() {
        let id1 = ViewId::new();
        let id2 = ViewId::new();
        let id3 = ViewId::new();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_bounds() {
        let bounds = Bounds::new(10, 20, 800, 600);
        assert_eq!(bounds.x, 10);
        assert_eq!(bounds.y, 20);
        assert_eq!(bounds.width, 800);
        assert_eq!(bounds.height, 600);
    }

    #[test]
    fn test_viewhost_creation() {
        let host = ViewHost::new();
        assert_eq!(host.view_count(), 0);
    }

    #[cfg(not(windows))]
    #[test]
    fn test_view_lifecycle_stub() {
        let host = ViewHost::new();
        let bounds = Bounds::new(0, 0, 800, 600);

        let view_id = host.create_view((), bounds).unwrap();
        assert_eq!(host.view_count(), 1);

        assert_eq!(host.get_bounds(view_id).unwrap(), bounds);

        let new_bounds = Bounds::new(10, 10, 1024, 768);
        host.set_bounds(view_id, new_bounds).unwrap();
        assert_eq!(host.get_bounds(view_id).unwrap(), new_bounds);

        host.destroy_view(view_id).unwrap();
        assert_eq!(host.view_count(), 0);
    }
}
