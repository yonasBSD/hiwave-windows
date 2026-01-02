//! Callback management for WebKit events
//!
//! This module provides type-safe callback registration for WebKit events.
//! It handles the conversion between Rust closures and C-style callbacks.

use std::ffi::c_void;
use webkit_wincairo_sys::*;

/// Decision for navigation events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDecision {
    /// Allow the navigation
    Allow,
    /// Cancel the navigation
    Cancel,
}

/// Trait for navigation handlers
/// Note: Send is not required because WebKit callbacks are called on the main UI thread
pub trait NavigationHandler:
    Fn(&str, WKNavigationType, bool) -> NavigationDecision + 'static
{
}
impl<F> NavigationHandler for F where
    F: Fn(&str, WKNavigationType, bool) -> NavigationDecision + 'static
{
}

/// Trait for title change handlers
pub trait TitleChangeHandler: Fn(&str) + 'static {}
impl<F> TitleChangeHandler for F where F: Fn(&str) + 'static {}

/// Trait for IPC message handlers
/// Note: Send is not required because WebKit callbacks are called on the main UI thread
pub trait IpcHandler: Fn(&str) + 'static {}
impl<F> IpcHandler for F where F: Fn(&str) + 'static {}

/// Trait for load start handlers
pub trait LoadStartHandler: Fn() + 'static {}
impl<F> LoadStartHandler for F where F: Fn() + 'static {}

/// Trait for load finish handlers
pub trait LoadFinishHandler: Fn() + 'static {}
impl<F> LoadFinishHandler for F where F: Fn() + 'static {}

/// Trait for progress change handlers
pub trait ProgressHandler: Fn(f64) + 'static {}
impl<F> ProgressHandler for F where F: Fn(f64) + 'static {}

/// Internal state for managing callbacks
pub(crate) struct CallbackState {
    navigation_handler: Option<Box<dyn NavigationHandler>>,
    title_handler: Option<Box<dyn TitleChangeHandler>>,
    ipc_handler: Option<Box<dyn IpcHandler>>,
    load_start_handler: Option<Box<dyn LoadStartHandler>>,
    load_finish_handler: Option<Box<dyn LoadFinishHandler>>,
    progress_handler: Option<Box<dyn ProgressHandler>>,

    // Client structs that we keep alive (must be kept in heap memory)
    navigation_client: Option<Box<WKPageNavigationClientV0>>,
    loader_client: Option<Box<WKPageLoaderClientV0>>,
    injected_bundle_client: Option<Box<WKPageInjectedBundleClientV0>>,
}

impl CallbackState {
    pub fn new() -> Self {
        Self {
            navigation_handler: None,
            title_handler: None,
            ipc_handler: None,
            load_start_handler: None,
            load_finish_handler: None,
            progress_handler: None,
            navigation_client: None,
            loader_client: None,
            injected_bundle_client: None,
        }
    }

    pub fn set_navigation_handler<F>(&mut self, page: WKPageRef, handler: F)
    where
        F: NavigationHandler,
    {
        self.navigation_handler = Some(Box::new(handler));

        // Only register once
        if self.navigation_client.is_some() {
            return;
        }

        let mut client = Box::new(WKPageNavigationClientV0::default());
        client.base.client_info = self as *const CallbackState as *const c_void;
        client.decide_policy_for_navigation_action = Some(navigation_trampoline);

        unsafe {
            WKPageSetPageNavigationClient(page, &client.base as *const WKClientBase);
        }

        self.navigation_client = Some(client);
        log::info!("Navigation client registered with correct struct layout");
    }

    pub fn set_title_change_handler<F>(&mut self, page: WKPageRef, handler: F)
    where
        F: TitleChangeHandler,
    {
        self.title_handler = Some(Box::new(handler));
        self.ensure_loader_client(page);
    }

    pub fn set_ipc_handler<F>(&mut self, page: WKPageRef, handler: F)
    where
        F: IpcHandler,
    {
        self.ipc_handler = Some(Box::new(handler));
        self.ensure_injected_bundle_client(page);
    }

    pub fn set_load_start_handler<F>(&mut self, page: WKPageRef, handler: F)
    where
        F: LoadStartHandler,
    {
        self.load_start_handler = Some(Box::new(handler));
        self.ensure_loader_client(page);
    }

    pub fn set_load_finish_handler<F>(&mut self, page: WKPageRef, handler: F)
    where
        F: LoadFinishHandler,
    {
        self.load_finish_handler = Some(Box::new(handler));
        self.ensure_loader_client(page);
    }

    pub fn set_progress_handler<F>(&mut self, page: WKPageRef, handler: F)
    where
        F: ProgressHandler,
    {
        self.progress_handler = Some(Box::new(handler));
        self.ensure_loader_client(page);
    }

    fn ensure_loader_client(&mut self, page: WKPageRef) {
        if self.loader_client.is_some() {
            return;
        }

        let mut client = Box::new(WKPageLoaderClientV0::default());
        client.base.client_info = self as *const CallbackState as *const c_void;
        client.did_receive_title_for_frame = Some(title_change_trampoline);
        client.did_start_provisional_load_for_frame = Some(load_start_trampoline);
        client.did_finish_load_for_frame = Some(load_finish_trampoline);
        client.did_change_progress = Some(progress_trampoline);

        unsafe {
            WKPageSetPageLoaderClient(page, &client.base as *const WKClientBase);
        }

        self.loader_client = Some(client);
        log::info!("Loader client registered with correct struct layout (24 callbacks)");
    }

    fn ensure_injected_bundle_client(&mut self, page: WKPageRef) {
        if self.injected_bundle_client.is_some() {
            return;
        }

        let mut client = Box::new(WKPageInjectedBundleClientV0::default());
        client.base.client_info = self as *const CallbackState as *const c_void;
        client.did_receive_message_from_injected_bundle = Some(ipc_trampoline);

        unsafe {
            WKPageSetPageInjectedBundleClient(page, &client.base as *const WKClientBase);
        }

        self.injected_bundle_client = Some(client);
        log::info!("Injected bundle client registered for IPC messages");
    }
}

// ============================================================================
// Trampoline Functions - These MUST match the callback signatures exactly
// ============================================================================

/// Navigation policy decision callback
/// Signature from WKPageNavigationClient.h:
/// void (*)(WKPageRef, WKNavigationActionRef, WKFramePolicyListenerRef, WKTypeRef, const void*)
unsafe extern "C" fn navigation_trampoline(
    _page: WKPageRef,
    navigation_action: WKNavigationActionRef,
    listener: WKFramePolicyListenerRef,
    _user_data: WKTypeRef,
    client_info: *const c_void,
) {
    if client_info.is_null() || navigation_action.is_null() || listener.is_null() {
        // Default: allow navigation
        if !listener.is_null() {
            WKFramePolicyListenerUse(listener);
        }
        return;
    }

    let state = &*(client_info as *const CallbackState);

    let decision = if let Some(ref handler) = state.navigation_handler {
        // Get URL from navigation action via request
        let request = WKNavigationActionCopyRequest(navigation_action);
        let url = if !request.is_null() {
            let wk_url = WKURLRequestCopyURL(request);
            let url_str = wk_url_to_string(wk_url).unwrap_or_default();
            if !wk_url.is_null() {
                WKRelease(wk_url);
            }
            WKRelease(request);
            url_str
        } else {
            String::new()
        };

        let nav_type = WKNavigationActionGetNavigationType(navigation_action);
        let user_gesture = WKNavigationActionHasUnconsumedUserGesture(navigation_action);

        handler(&url, nav_type, user_gesture)
    } else {
        NavigationDecision::Allow
    };

    // Use the frame policy listener to communicate the decision
    match decision {
        NavigationDecision::Allow => WKFramePolicyListenerUse(listener),
        NavigationDecision::Cancel => WKFramePolicyListenerIgnore(listener),
    }
}

/// Title change callback
/// Signature from WKPageLoaderClient.h:
/// void (*)(WKPageRef, WKStringRef, WKFrameRef, WKTypeRef, const void*)
unsafe extern "C" fn title_change_trampoline(
    _page: WKPageRef,
    title: WKStringRef,
    frame: WKFrameRef,
    _user_data: WKTypeRef,
    client_info: *const c_void,
) {
    if client_info.is_null() || title.is_null() {
        return;
    }

    // Only handle title changes for the main frame
    if !WKFrameIsMainFrame(frame) {
        return;
    }

    let state = &*(client_info as *const CallbackState);

    if let Some(ref handler) = state.title_handler {
        if let Some(title_str) = wk_string_to_string(title) {
            handler(&title_str);
        }
    }
}

/// Load start callback
/// Signature from WKPageLoaderClient.h:
/// void (*)(WKPageRef, WKFrameRef, WKTypeRef, const void*)
unsafe extern "C" fn load_start_trampoline(
    _page: WKPageRef,
    frame: WKFrameRef,
    _user_data: WKTypeRef,
    client_info: *const c_void,
) {
    if client_info.is_null() {
        return;
    }

    // Only handle main frame
    if !WKFrameIsMainFrame(frame) {
        return;
    }

    let state = &*(client_info as *const CallbackState);

    if let Some(ref handler) = state.load_start_handler {
        handler();
    }
}

/// Load finish callback
/// Signature from WKPageLoaderClient.h:
/// void (*)(WKPageRef, WKFrameRef, WKTypeRef, const void*)
unsafe extern "C" fn load_finish_trampoline(
    _page: WKPageRef,
    frame: WKFrameRef,
    _user_data: WKTypeRef,
    client_info: *const c_void,
) {
    if client_info.is_null() {
        return;
    }

    // Only handle main frame
    if !WKFrameIsMainFrame(frame) {
        return;
    }

    let state = &*(client_info as *const CallbackState);

    if let Some(ref handler) = state.load_finish_handler {
        handler();
    }
}

/// Progress change callback
/// Signature from WKPageLoaderClient.h:
/// void (*)(WKPageRef, const void*)
unsafe extern "C" fn progress_trampoline(page: WKPageRef, client_info: *const c_void) {
    if client_info.is_null() {
        return;
    }

    let state = &*(client_info as *const CallbackState);

    if let Some(ref handler) = state.progress_handler {
        let progress = WKPageGetEstimatedProgress(page);
        handler(progress);
    }
}

/// IPC message callback (from injected bundle)
/// Signature from WKPageInjectedBundleClient.h:
/// void (*)(WKPageRef, WKStringRef, WKTypeRef, const void*)
unsafe extern "C" fn ipc_trampoline(
    _page: WKPageRef,
    message_name: WKStringRef,
    message_body: WKTypeRef,
    client_info: *const c_void,
) {
    if client_info.is_null() || message_name.is_null() {
        return;
    }

    let state = &*(client_info as *const CallbackState);

    if let Some(ref handler) = state.ipc_handler {
        // For now, just pass the message name. A full implementation would
        // also handle the message body.
        if let Some(name) = wk_string_to_string(message_name) {
            // If the body is a string, try to get it
            let body = if !message_body.is_null() {
                wk_string_to_string(message_body as WKStringRef)
            } else {
                None
            };

            let message = body.unwrap_or(name);
            handler(&message);
        }
    }
}
