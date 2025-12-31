//! WebKit context management
//!
//! The WebKitContext manages shared resources like the process pool,
//! cookie storage, and caches. Multiple views can share a single context.

use std::sync::Arc;
use webkit_wincairo_sys::*;
use crate::error::{Result, WebKitError};

/// A WebKit browsing context
///
/// The context manages shared resources for all WebViews. You typically
/// create one context and share it among multiple views.
///
/// # Thread Safety
///
/// WebKitContext is thread-safe and can be shared across threads using `Arc`.
/// However, most WebKit operations must be performed on the main/UI thread.
pub struct WebKitContext {
    raw: WKContextRef,
    page_group: WKPageGroupRef,
}

// WebKitContext is Send + Sync because the underlying WebKit handles
// thread safety internally for context-level operations
unsafe impl Send for WebKitContext {}
unsafe impl Sync for WebKitContext {}

impl WebKitContext {
    /// Create a new WebKit context
    ///
    /// This creates an isolated browsing context with its own process pool,
    /// cookie storage, and cache.
    pub fn new() -> Result<Arc<Self>> {
        unsafe {
            let raw = WKContextCreate();
            if raw.is_null() {
                return Err(WebKitError::ContextCreationFailed);
            }

            // Create a page group for this context
            let group_id = wk_string_from_str("HiWavePageGroup");
            let page_group = WKPageGroupCreateWithIdentifier(group_id);
            WKRelease(group_id);

            if page_group.is_null() {
                WKRelease(raw);
                return Err(WebKitError::ContextCreationFailed);
            }

            Ok(Arc::new(Self { raw, page_group }))
        }
    }

    /// Create a context with a custom storage path
    ///
    /// This allows you to specify where cookies, caches, and other
    /// persistent data should be stored.
    pub fn with_storage_path(path: &str) -> Result<Arc<Self>> {
        unsafe {
            let path_str = wk_string_from_str(path);
            let raw = WKContextCreateWithConfiguration(path_str);
            WKRelease(path_str);

            if raw.is_null() {
                return Err(WebKitError::ContextCreationFailed);
            }

            let group_id = wk_string_from_str("HiWavePageGroup");
            let page_group = WKPageGroupCreateWithIdentifier(group_id);
            WKRelease(group_id);

            if page_group.is_null() {
                WKRelease(raw);
                return Err(WebKitError::ContextCreationFailed);
            }

            Ok(Arc::new(Self { raw, page_group }))
        }
    }

    /// Get the raw WKContextRef
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid for the lifetime of this context.
    pub(crate) fn raw(&self) -> WKContextRef {
        self.raw
    }

    /// Get the page group for this context
    pub(crate) fn page_group(&self) -> WKPageGroupRef {
        self.page_group
    }
}

impl Drop for WebKitContext {
    fn drop(&mut self) {
        unsafe {
            if !self.page_group.is_null() {
                WKRelease(self.page_group);
            }
            if !self.raw.is_null() {
                WKRelease(self.raw);
            }
        }
    }
}
