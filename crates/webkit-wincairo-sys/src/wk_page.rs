//! WebKit page functions
//!
//! WKPage represents a web page. It provides methods for navigation,
//! JavaScript execution, and handling page events.

use std::ffi::c_void;
use super::wk_types::*;
use super::wk_callbacks::*;
use windows_sys::Win32::Foundation::BOOL;

extern "C" {
    // ========== Navigation ==========

    /// Load a URL in the page
    pub fn WKPageLoadURL(page: WKPageRef, url: WKURLRef);

    /// Load a URL request
    pub fn WKPageLoadURLRequest(page: WKPageRef, url_request: WKTypeRef);

    /// Load HTML string with a base URL
    pub fn WKPageLoadHTMLString(
        page: WKPageRef,
        html_string: WKStringRef,
        base_url: WKURLRef,
    );

    /// Load plain text
    pub fn WKPageLoadPlainTextString(page: WKPageRef, text: WKStringRef);

    /// Reload the page
    pub fn WKPageReload(page: WKPageRef);

    /// Reload the page, revalidating content data
    pub fn WKPageReloadFromOrigin(page: WKPageRef);

    /// Stop loading
    pub fn WKPageStopLoading(page: WKPageRef);

    /// Go back in history
    pub fn WKPageGoBack(page: WKPageRef);

    /// Can we go back?
    pub fn WKPageCanGoBack(page: WKPageRef) -> bool;

    /// Go forward in history
    pub fn WKPageGoForward(page: WKPageRef);

    /// Can we go forward?
    pub fn WKPageCanGoForward(page: WKPageRef) -> bool;

    /// Get the back-forward list
    pub fn WKPageGetBackForwardList(page: WKPageRef) -> WKBackForwardListRef;

    /// Go to a specific back-forward list item
    pub fn WKPageGoToBackForwardListItem(
        page: WKPageRef,
        item: WKBackForwardListItemRef,
    );

    // ========== Page Info ==========

    /// Get the main frame
    pub fn WKPageGetMainFrame(page: WKPageRef) -> WKFrameRef;

    /// Get the page URL
    pub fn WKPageCopyURL(page: WKPageRef) -> WKURLRef;

    /// Get the page title
    pub fn WKPageCopyTitle(page: WKPageRef) -> WKStringRef;

    /// Get the estimated loading progress (0.0 to 1.0)
    pub fn WKPageGetEstimatedProgress(page: WKPageRef) -> f64;

    /// Check if the page is loading
    pub fn WKPageIsLoading(page: WKPageRef) -> bool;

    // ========== JavaScript ==========

    /// Run JavaScript in the main frame
    /// The callback will be called with the result
    pub fn WKPageRunJavaScriptInMainFrame(
        page: WKPageRef,
        script: WKStringRef,
        context: *mut c_void,
        callback: WKPageRunJavaScriptFunction,
    );

    /// Run JavaScript in the main frame (Promise-based, newer API)
    pub fn WKPageRunJavaScriptInMainFrameForReply(
        page: WKPageRef,
        script: WKStringRef,
        context: *mut c_void,
        callback: WKPageRunJavaScriptFunction,
    );

    // ========== Client Setup ==========

    /// Set the page navigation client (handles navigation events)
    pub fn WKPageSetPageNavigationClient(
        page: WKPageRef,
        client: *const WKPageNavigationClientV0,
    );

    /// Set the page loader client (handles load events)
    pub fn WKPageSetPageLoaderClient(
        page: WKPageRef,
        client: *const WKPageLoaderClientV0,
    );

    /// Set the page UI client (handles UI events like alerts, new windows)
    pub fn WKPageSetPageUIClient(
        page: WKPageRef,
        client: *const WKPageUIClientV0,
    );

    /// Set the page policy client (handles navigation decisions)
    pub fn WKPageSetPagePolicyClient(
        page: WKPageRef,
        client: *const WKPagePolicyClientV0,
    );

    // ========== Settings ==========

    /// Get the page's preferences
    pub fn WKPageGetPreferences(page: WKPageRef) -> WKTypeRef;

    /// Set whether JavaScript is enabled
    pub fn WKPreferencesSetJavaScriptEnabled(
        preferences: WKTypeRef,
        enabled: bool,
    );

    /// Check if JavaScript is enabled
    pub fn WKPreferencesGetJavaScriptEnabled(preferences: WKTypeRef) -> bool;

    /// Set whether developer extras (inspector) are enabled
    pub fn WKPreferencesSetDeveloperExtrasEnabled(
        preferences: WKTypeRef,
        enabled: bool,
    );

    /// Set whether local storage is enabled
    pub fn WKPreferencesSetLocalStorageEnabled(
        preferences: WKTypeRef,
        enabled: bool,
    );

    // ========== Zoom ==========

    /// Set the page zoom factor
    pub fn WKPageSetPageZoomFactor(page: WKPageRef, zoom_factor: f64);

    /// Get the page zoom factor
    pub fn WKPageGetPageZoomFactor(page: WKPageRef) -> f64;

    /// Set the text zoom factor
    pub fn WKPageSetTextZoomFactor(page: WKPageRef, zoom_factor: f64);

    /// Get the text zoom factor
    pub fn WKPageGetTextZoomFactor(page: WKPageRef) -> f64;

    // ========== Find ==========

    /// Find text in the page
    pub fn WKPageFindString(
        page: WKPageRef,
        string: WKStringRef,
        find_options: u32,
        max_match_count: u32,
    );

    /// Hide find UI
    pub fn WKPageHideFindUI(page: WKPageRef);

    /// Count matches for a string
    pub fn WKPageCountStringMatches(
        page: WKPageRef,
        string: WKStringRef,
        find_options: u32,
        max_match_count: u32,
    );

    // ========== Frame Info ==========

    /// Check if a frame is the main frame
    pub fn WKFrameIsMainFrame(frame: WKFrameRef) -> bool;

    /// Get the URL of a frame
    pub fn WKFrameCopyURL(frame: WKFrameRef) -> WKURLRef;

    /// Get the provisional URL of a frame (during navigation)
    pub fn WKFrameCopyProvisionalURL(frame: WKFrameRef) -> WKURLRef;

    // ========== Back/Forward List ==========

    /// Get the current item in the back-forward list
    pub fn WKBackForwardListGetCurrentItem(
        list: WKBackForwardListRef,
    ) -> WKBackForwardListItemRef;

    /// Get the back item
    pub fn WKBackForwardListGetBackItem(
        list: WKBackForwardListRef,
    ) -> WKBackForwardListItemRef;

    /// Get the forward item
    pub fn WKBackForwardListGetForwardItem(
        list: WKBackForwardListRef,
    ) -> WKBackForwardListItemRef;

    /// Get the URL of a back-forward list item
    pub fn WKBackForwardListItemCopyURL(
        item: WKBackForwardListItemRef,
    ) -> WKURLRef;

    /// Get the title of a back-forward list item
    pub fn WKBackForwardListItemCopyTitle(
        item: WKBackForwardListItemRef,
    ) -> WKStringRef;

    // ========== Clipboard ==========

    /// Execute an editing command (like "copy", "paste", "selectAll")
    pub fn WKPageExecuteEditingCommand(
        page: WKPageRef,
        command_name: WKStringRef,
        argument: WKStringRef,
    );

    /// Check if an editing command can be executed
    pub fn WKPageCanExecuteEditingCommand(
        page: WKPageRef,
        command_name: WKStringRef,
    ) -> BOOL;

    // ========== Print ==========

    /// Print the page (Windows-specific)
    pub fn WKPagePrint(page: WKPageRef);
}

/// Find options flags
pub mod find_options {
    pub const WK_FIND_OPTIONS_CASE_INSENSITIVE: u32 = 1 << 0;
    pub const WK_FIND_OPTIONS_AT_WORD_STARTS: u32 = 1 << 1;
    pub const WK_FIND_OPTIONS_TREAT_MEDIA_CAPTIONS_AS_TEXT: u32 = 1 << 2;
    pub const WK_FIND_OPTIONS_BACKWARDS: u32 = 1 << 3;
    pub const WK_FIND_OPTIONS_WRAP_AROUND: u32 = 1 << 4;
    pub const WK_FIND_OPTIONS_SHOW_OVERLAY: u32 = 1 << 5;
    pub const WK_FIND_OPTIONS_SHOW_FIND_INDICATOR: u32 = 1 << 6;
    pub const WK_FIND_OPTIONS_SHOW_HIGHLIGHT: u32 = 1 << 7;
}
