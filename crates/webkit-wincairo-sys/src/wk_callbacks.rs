//! WebKit callback types and client structures
//!
//! WebKit uses a client-based callback system where you register callback
//! functions for various events. This module defines the callback function
//! types and client structures.

use std::ffi::c_void;
use super::wk_types::*;

/// Base client structure - all clients have version and clientInfo
#[repr(C)]
pub struct WKClientBase {
    pub version: u32,
    pub client_info: *mut c_void,
}

// ============================================================================
// Callback Function Types
// ============================================================================

/// Callback for JavaScript execution results
pub type WKPageRunJavaScriptFunction = Option<
    unsafe extern "C" fn(
        result: WKSerializedScriptValueRef,
        error: WKErrorRef,
        context: *mut c_void,
    ),
>;

/// Callback for navigation action policy decisions
pub type WKPageNavigationDecidePolicyForNavigationActionCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        navigation_action: WKNavigationActionRef,
        client_info: *mut c_void,
    ) -> WKNavigationActionPolicy,
>;

/// Callback for navigation response policy decisions
pub type WKPageNavigationDecidePolicyForResponseCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        response: WKNavigationResponseRef,
        client_info: *mut c_void,
    ) -> WKNavigationResponsePolicy,
>;

/// Callback for when navigation starts
pub type WKPageNavigationDidStartProvisionalNavigationCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        navigation: WKNavigationRef,
        user_data: WKTypeRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for when navigation receives a server redirect
pub type WKPageNavigationDidReceiveServerRedirectCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        navigation: WKNavigationRef,
        user_data: WKTypeRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for when navigation commits
pub type WKPageNavigationDidCommitNavigationCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        navigation: WKNavigationRef,
        user_data: WKTypeRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for when navigation finishes
pub type WKPageNavigationDidFinishNavigationCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        navigation: WKNavigationRef,
        user_data: WKTypeRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for when navigation fails
pub type WKPageNavigationDidFailNavigationCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        navigation: WKNavigationRef,
        error: WKErrorRef,
        user_data: WKTypeRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for title changes
pub type WKPageLoaderDidReceiveTitleForFrameCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        title: WKStringRef,
        frame: WKFrameRef,
        user_data: WKTypeRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for progress changes
pub type WKPageLoaderDidChangeProgressCallback = Option<
    unsafe extern "C" fn(page: WKPageRef, client_info: *mut c_void),
>;

/// Callback for new window requests
pub type WKPageUICreateNewPageCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        configuration: WKPageConfigurationRef,
        navigation_action: WKNavigationActionRef,
        client_info: *mut c_void,
    ) -> WKPageRef,
>;

/// Callback for close window requests
pub type WKPageUICloseCallback = Option<
    unsafe extern "C" fn(page: WKPageRef, client_info: *mut c_void),
>;

/// Callback for JavaScript alert()
pub type WKPageUIRunJavaScriptAlertCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        message: WKStringRef,
        frame: WKFrameRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for JavaScript confirm()
pub type WKPageUIRunJavaScriptConfirmCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        message: WKStringRef,
        frame: WKFrameRef,
        client_info: *mut c_void,
    ) -> bool,
>;

/// Callback for JavaScript prompt()
pub type WKPageUIRunJavaScriptPromptCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        message: WKStringRef,
        default_value: WKStringRef,
        frame: WKFrameRef,
        client_info: *mut c_void,
    ) -> WKStringRef,
>;

/// Callback for IPC messages from JavaScript
pub type WKPageUIDidReceiveMessageFromInjectedBundleCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        message_name: WKStringRef,
        message_body: WKTypeRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for download requests
pub type WKPageUIDecideDestinationWithSuggestedFilenameCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        download: WKDownloadRef,
        suggested_filename: WKStringRef,
        client_info: *mut c_void,
    ) -> WKStringRef,
>;

/// Callback for download progress
pub type WKPageUIDidReceiveDataForDownloadCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        download: WKDownloadRef,
        length: u64,
        client_info: *mut c_void,
    ),
>;

/// Callback for download completion
pub type WKPageUIDidFinishDownloadCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        download: WKDownloadRef,
        client_info: *mut c_void,
    ),
>;

/// Callback for download failure
pub type WKPageUIDidFailDownloadCallback = Option<
    unsafe extern "C" fn(
        page: WKPageRef,
        download: WKDownloadRef,
        error: WKErrorRef,
        client_info: *mut c_void,
    ),
>;

// ============================================================================
// Client Structures
// ============================================================================

/// Navigation client - handles navigation-related events
#[repr(C)]
pub struct WKPageNavigationClientV0 {
    pub base: WKClientBase,
    pub decide_policy_for_navigation_action:
        WKPageNavigationDecidePolicyForNavigationActionCallback,
    pub decide_policy_for_response: WKPageNavigationDecidePolicyForResponseCallback,
    pub did_start_provisional_navigation:
        WKPageNavigationDidStartProvisionalNavigationCallback,
    pub did_receive_server_redirect: WKPageNavigationDidReceiveServerRedirectCallback,
    pub did_commit_navigation: WKPageNavigationDidCommitNavigationCallback,
    pub did_finish_navigation: WKPageNavigationDidFinishNavigationCallback,
    pub did_fail_navigation: WKPageNavigationDidFailNavigationCallback,
    pub did_fail_provisional_navigation: WKPageNavigationDidFailNavigationCallback,
}

impl Default for WKPageNavigationClientV0 {
    fn default() -> Self {
        Self {
            base: WKClientBase {
                version: 0,
                client_info: std::ptr::null_mut(),
            },
            decide_policy_for_navigation_action: None,
            decide_policy_for_response: None,
            did_start_provisional_navigation: None,
            did_receive_server_redirect: None,
            did_commit_navigation: None,
            did_finish_navigation: None,
            did_fail_navigation: None,
            did_fail_provisional_navigation: None,
        }
    }
}

/// Loader client - handles page load events
#[repr(C)]
pub struct WKPageLoaderClientV0 {
    pub base: WKClientBase,
    pub did_start_provisional_load_for_frame: Option<
        unsafe extern "C" fn(
            page: WKPageRef,
            frame: WKFrameRef,
            user_data: WKTypeRef,
            client_info: *mut c_void,
        ),
    >,
    pub did_receive_server_redirect_for_provisional_load_for_frame: Option<
        unsafe extern "C" fn(
            page: WKPageRef,
            frame: WKFrameRef,
            user_data: WKTypeRef,
            client_info: *mut c_void,
        ),
    >,
    pub did_fail_provisional_load_with_error_for_frame: Option<
        unsafe extern "C" fn(
            page: WKPageRef,
            frame: WKFrameRef,
            error: WKErrorRef,
            user_data: WKTypeRef,
            client_info: *mut c_void,
        ),
    >,
    pub did_commit_load_for_frame: Option<
        unsafe extern "C" fn(
            page: WKPageRef,
            frame: WKFrameRef,
            user_data: WKTypeRef,
            client_info: *mut c_void,
        ),
    >,
    pub did_finish_document_load_for_frame: Option<
        unsafe extern "C" fn(
            page: WKPageRef,
            frame: WKFrameRef,
            user_data: WKTypeRef,
            client_info: *mut c_void,
        ),
    >,
    pub did_finish_load_for_frame: Option<
        unsafe extern "C" fn(
            page: WKPageRef,
            frame: WKFrameRef,
            user_data: WKTypeRef,
            client_info: *mut c_void,
        ),
    >,
    pub did_fail_load_with_error_for_frame: Option<
        unsafe extern "C" fn(
            page: WKPageRef,
            frame: WKFrameRef,
            error: WKErrorRef,
            user_data: WKTypeRef,
            client_info: *mut c_void,
        ),
    >,
    pub did_receive_title_for_frame: WKPageLoaderDidReceiveTitleForFrameCallback,
    pub did_change_progress: WKPageLoaderDidChangeProgressCallback,
}

impl Default for WKPageLoaderClientV0 {
    fn default() -> Self {
        Self {
            base: WKClientBase {
                version: 0,
                client_info: std::ptr::null_mut(),
            },
            did_start_provisional_load_for_frame: None,
            did_receive_server_redirect_for_provisional_load_for_frame: None,
            did_fail_provisional_load_with_error_for_frame: None,
            did_commit_load_for_frame: None,
            did_finish_document_load_for_frame: None,
            did_finish_load_for_frame: None,
            did_fail_load_with_error_for_frame: None,
            did_receive_title_for_frame: None,
            did_change_progress: None,
        }
    }
}

/// UI client - handles UI events (alerts, new windows, etc.)
#[repr(C)]
pub struct WKPageUIClientV0 {
    pub base: WKClientBase,
    pub create_new_page: WKPageUICreateNewPageCallback,
    pub close: WKPageUICloseCallback,
    pub run_javascript_alert: WKPageUIRunJavaScriptAlertCallback,
    pub run_javascript_confirm: WKPageUIRunJavaScriptConfirmCallback,
    pub run_javascript_prompt: WKPageUIRunJavaScriptPromptCallback,
    pub did_receive_message_from_injected_bundle:
        WKPageUIDidReceiveMessageFromInjectedBundleCallback,
}

impl Default for WKPageUIClientV0 {
    fn default() -> Self {
        Self {
            base: WKClientBase {
                version: 0,
                client_info: std::ptr::null_mut(),
            },
            create_new_page: None,
            close: None,
            run_javascript_alert: None,
            run_javascript_confirm: None,
            run_javascript_prompt: None,
            did_receive_message_from_injected_bundle: None,
        }
    }
}

/// Policy client - handles navigation policy decisions
#[repr(C)]
pub struct WKPagePolicyClientV0 {
    pub base: WKClientBase,
    pub decide_policy_for_navigation_action:
        WKPageNavigationDecidePolicyForNavigationActionCallback,
    pub decide_policy_for_response: WKPageNavigationDecidePolicyForResponseCallback,
}

impl Default for WKPagePolicyClientV0 {
    fn default() -> Self {
        Self {
            base: WKClientBase {
                version: 0,
                client_info: std::ptr::null_mut(),
            },
            decide_policy_for_navigation_action: None,
            decide_policy_for_response: None,
        }
    }
}

// ============================================================================
// Helper Functions for Navigation Actions
// ============================================================================

extern "C" {
    /// Get the URL from a navigation action
    pub fn WKNavigationActionCopyURL(
        navigation_action: WKNavigationActionRef,
    ) -> WKURLRef;

    /// Get the navigation type from a navigation action
    pub fn WKNavigationActionGetNavigationType(
        navigation_action: WKNavigationActionRef,
    ) -> WKNavigationType;

    /// Check if the navigation was user-initiated
    pub fn WKNavigationActionGetWasUserInitiated(
        navigation_action: WKNavigationActionRef,
    ) -> bool;

    /// Get the source frame of the navigation
    pub fn WKNavigationActionGetSourceFrame(
        navigation_action: WKNavigationActionRef,
    ) -> WKFrameRef;

    /// Get the target frame of the navigation (may be null for new window)
    pub fn WKNavigationActionGetDestinationFrame(
        navigation_action: WKNavigationActionRef,
    ) -> WKFrameRef;
}
