//! WebKit context management
//!
//! The WebKitContext manages shared resources like the process pool,
//! cookie storage, and caches. Multiple views can share a single context.

use crate::error::{Result, WebKitError};
use std::sync::Arc;
use webkit_wincairo_sys::*;

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
    page_group: Option<WKPageGroupRef>,
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
            log::info!("Creating WebKit context configuration...");

            // Create a configuration first (more reliable than WKContextCreate)
            let config = WKContextConfigurationCreate();
            log::info!("WKContextConfigurationCreate returned: {:p}", config);
            if config.is_null() {
                log::error!("WKContextConfigurationCreate returned NULL");
                return Err(WebKitError::ContextCreationFailed);
            }

            // Set injected bundle path (required for script message handlers to work)
            // Find the bundle DLL in the same directory as the executable
            let bundle_path = if let Ok(exe_path) = std::env::current_exe() {
                if let Some(dir) = exe_path.parent() {
                    let bundle = dir.join("MiniBrowserInjectedBundle.dll");
                    if bundle.exists() {
                        Some(bundle.to_string_lossy().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(ref path) = bundle_path {
                log::info!("Setting injected bundle path: {}", path);
                let wk_path = wk_string_from_str(path);
                WKContextConfigurationSetInjectedBundlePath(config, wk_path);
                WKRelease(wk_path);
            } else {
                log::warn!("MiniBrowserInjectedBundle.dll not found - script message handlers may not work");
            }

            log::info!("Creating WebKit context from configuration...");
            // Create context with the configuration
            let raw = WKContextCreateWithConfiguration(config);
            log::info!("WKContextCreateWithConfiguration returned: {:p}", raw);
            WKRelease(config);

            if raw.is_null() {
                log::error!("WKContextCreateWithConfiguration returned NULL");
                return Err(WebKitError::ContextCreationFailed);
            }

            // Try to create a page group for this context (optional)
            log::info!("Creating page group...");
            let group_id = wk_string_from_str("HiWavePageGroup");
            let page_group = WKPageGroupCreateWithIdentifier(group_id);
            log::info!("WKPageGroupCreateWithIdentifier returned: {:p}", page_group);
            WKRelease(group_id);

            let page_group = if page_group.is_null() {
                log::warn!(
                    "WKPageGroupCreateWithIdentifier returned NULL - using default page group"
                );
                None
            } else {
                Some(page_group)
            };

            log::info!("WebKit context created successfully!");
            Ok(Arc::new(Self { raw, page_group }))
        }
    }

    /// Create a context with a custom injected bundle path
    ///
    /// This allows you to specify a WebKit injected bundle to use.
    #[allow(dead_code)]
    pub fn with_bundle_path(path: &str) -> Result<Arc<Self>> {
        unsafe {
            let path_str = wk_string_from_str(path);
            let raw = WKContextCreateWithInjectedBundlePath(path_str);
            WKRelease(path_str);

            if raw.is_null() {
                return Err(WebKitError::ContextCreationFailed);
            }

            let group_id = wk_string_from_str("HiWavePageGroup");
            let page_group = WKPageGroupCreateWithIdentifier(group_id);
            WKRelease(group_id);

            let page_group = if page_group.is_null() {
                None
            } else {
                Some(page_group)
            };

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

    /// Get the page group for this context (if available)
    pub(crate) fn page_group(&self) -> WKPageGroupRef {
        self.page_group.unwrap_or(std::ptr::null_mut())
    }
}

impl Drop for WebKitContext {
    fn drop(&mut self) {
        unsafe {
            if let Some(page_group) = self.page_group {
                if !page_group.is_null() {
                    WKRelease(page_group);
                }
            }
            if !self.raw.is_null() {
                WKRelease(self.raw);
            }
        }
    }
}
