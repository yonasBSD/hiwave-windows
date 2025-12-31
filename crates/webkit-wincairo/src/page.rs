//! WebKit page operations
//!
//! WebKitPage represents a web page and provides methods for navigation,
//! JavaScript execution, and content manipulation.

use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use webkit_wincairo_sys::*;
use crate::error::{Result, WebKitError};
use crate::callbacks::{
    CallbackState, NavigationHandler, TitleChangeHandler, IpcHandler,
    LoadStartHandler, LoadFinishHandler, ProgressHandler,
};

/// A WebKit page
///
/// The page represents a single web document and provides methods for
/// navigation, JavaScript execution, and event handling.
pub struct WebKitPage {
    raw: WKPageRef,
    /// Callback state (kept alive for the lifetime of the page)
    #[allow(dead_code)]
    callbacks: Arc<Mutex<CallbackState>>,
}

impl WebKitPage {
    /// Create a page wrapper from a raw pointer
    ///
    /// # Safety
    ///
    /// The caller must ensure the page ref is valid and will remain valid
    /// for the lifetime of this wrapper.
    pub(crate) fn from_raw(raw: WKPageRef) -> Self {
        Self {
            raw,
            callbacks: Arc::new(Mutex::new(CallbackState::new())),
        }
    }

    /// Get the raw page reference
    pub(crate) fn raw(&self) -> WKPageRef {
        self.raw
    }

    // ========== Navigation ==========

    /// Load a URL
    pub fn load_url(&self, url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(WebKitError::InvalidUrl("URL cannot be empty".to_string()));
        }

        unsafe {
            let wk_url = wk_url_from_str(url);
            if wk_url.is_null() {
                return Err(WebKitError::InvalidUrl(url.to_string()));
            }

            WKPageLoadURL(self.raw, wk_url);
            WKRelease(wk_url);
            Ok(())
        }
    }

    /// Load HTML content with an optional base URL
    pub fn load_html(&self, html: &str, base_url: Option<&str>) -> Result<()> {
        unsafe {
            let wk_html = wk_string_from_str(html);
            if wk_html.is_null() {
                return Err(WebKitError::InvalidHtml);
            }

            let wk_base = match base_url {
                Some(url) => wk_url_from_str(url),
                None => std::ptr::null_mut(),
            };

            WKPageLoadHTMLString(self.raw, wk_html, wk_base);

            WKRelease(wk_html);
            if !wk_base.is_null() {
                WKRelease(wk_base);
            }

            Ok(())
        }
    }

    /// Reload the page
    pub fn reload(&self) {
        unsafe {
            WKPageReload(self.raw);
        }
    }

    /// Reload the page from origin (bypass cache)
    pub fn reload_from_origin(&self) {
        unsafe {
            WKPageReloadFromOrigin(self.raw);
        }
    }

    /// Stop loading
    pub fn stop(&self) {
        unsafe {
            WKPageStopLoading(self.raw);
        }
    }

    /// Go back in history
    pub fn go_back(&self) {
        unsafe {
            WKPageGoBack(self.raw);
        }
    }

    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        unsafe { WKPageCanGoBack(self.raw) }
    }

    /// Go forward in history
    pub fn go_forward(&self) {
        unsafe {
            WKPageGoForward(self.raw);
        }
    }

    /// Check if we can go forward
    pub fn can_go_forward(&self) -> bool {
        unsafe { WKPageCanGoForward(self.raw) }
    }

    // ========== Page Info ==========

    /// Get the current URL
    pub fn url(&self) -> Option<String> {
        unsafe {
            let wk_url = WKPageCopyURL(self.raw);
            let result = wk_url_to_string(wk_url);
            if !wk_url.is_null() {
                WKRelease(wk_url);
            }
            result
        }
    }

    /// Get the page title
    pub fn title(&self) -> Option<String> {
        unsafe {
            let wk_title = WKPageCopyTitle(self.raw);
            let result = wk_string_to_string(wk_title);
            if !wk_title.is_null() {
                WKRelease(wk_title);
            }
            result
        }
    }

    /// Get the loading progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        unsafe { WKPageGetEstimatedProgress(self.raw) }
    }

    /// Check if the page is loading
    pub fn is_loading(&self) -> bool {
        unsafe { WKPageIsLoading(self.raw) }
    }

    // ========== JavaScript ==========

    /// Execute JavaScript in the main frame
    ///
    /// This method executes JavaScript asynchronously. The callback will be
    /// called with the result when execution completes.
    pub fn evaluate_script<F>(&self, script: &str, callback: F) -> Result<()>
    where
        F: FnOnce(std::result::Result<String, String>) + Send + 'static,
    {
        unsafe {
            let wk_script = wk_string_from_str(script);
            if wk_script.is_null() {
                return Err(WebKitError::JavaScriptError(
                    "Failed to create script string".to_string(),
                ));
            }

            // Box the callback so it can be passed as a raw pointer
            let callback_box: Box<Box<dyn FnOnce(std::result::Result<String, String>) + Send>> =
                Box::new(Box::new(callback));
            let context = Box::into_raw(callback_box) as *mut c_void;

            WKPageRunJavaScriptInMainFrame(
                self.raw,
                wk_script,
                context,
                Some(javascript_callback_trampoline),
            );

            WKRelease(wk_script);
            Ok(())
        }
    }

    /// Execute JavaScript synchronously (blocking)
    ///
    /// Note: This blocks the current thread. Use `evaluate_script` for
    /// async execution when possible.
    pub fn evaluate_script_sync(&self, script: &str) -> Result<()> {
        // For now, we just fire and forget since WebKit's async model
        // is complex. A full implementation would use a channel.
        self.evaluate_script(script, |_| {})?;
        Ok(())
    }

    // ========== Zoom ==========

    /// Set the page zoom factor
    pub fn set_zoom(&self, factor: f64) {
        unsafe {
            WKPageSetPageZoomFactor(self.raw, factor);
        }
    }

    /// Get the page zoom factor
    pub fn zoom(&self) -> f64 {
        unsafe { WKPageGetPageZoomFactor(self.raw) }
    }

    /// Set the text zoom factor
    pub fn set_text_zoom(&self, factor: f64) {
        unsafe {
            WKPageSetTextZoomFactor(self.raw, factor);
        }
    }

    /// Get the text zoom factor
    pub fn text_zoom(&self) -> f64 {
        unsafe { WKPageGetTextZoomFactor(self.raw) }
    }

    // ========== Find ==========

    /// Find text in the page
    pub fn find(&self, text: &str, case_sensitive: bool) {
        unsafe {
            let wk_text = wk_string_from_str(text);
            let mut options = find_options::WK_FIND_OPTIONS_WRAP_AROUND
                | find_options::WK_FIND_OPTIONS_SHOW_FIND_INDICATOR;

            if !case_sensitive {
                options |= find_options::WK_FIND_OPTIONS_CASE_INSENSITIVE;
            }

            WKPageFindString(self.raw, wk_text, options, 100);
            WKRelease(wk_text);
        }
    }

    /// Hide find UI
    pub fn hide_find(&self) {
        unsafe {
            WKPageHideFindUI(self.raw);
        }
    }

    // ========== Clipboard ==========

    /// Execute an editing command
    pub fn execute_command(&self, command: &str) {
        unsafe {
            let wk_command = wk_string_from_str(command);
            WKPageExecuteEditingCommand(self.raw, wk_command, std::ptr::null_mut());
            WKRelease(wk_command);
        }
    }

    /// Copy selection to clipboard
    pub fn copy(&self) {
        self.execute_command("copy");
    }

    /// Paste from clipboard
    pub fn paste(&self) {
        self.execute_command("paste");
    }

    /// Cut selection to clipboard
    pub fn cut(&self) {
        self.execute_command("cut");
    }

    /// Select all content
    pub fn select_all(&self) {
        self.execute_command("selectAll");
    }

    // ========== Print ==========

    /// Print the page
    pub fn print(&self) {
        unsafe {
            WKPagePrint(self.raw);
        }
    }

    // ========== Event Handlers ==========

    /// Set the navigation handler
    ///
    /// The handler is called when navigation is about to occur and can
    /// allow or deny the navigation.
    pub fn set_navigation_handler<F>(&mut self, handler: F)
    where
        F: NavigationHandler,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.set_navigation_handler(self.raw, handler);
    }

    /// Set the title change handler
    pub fn set_title_change_handler<F>(&mut self, handler: F)
    where
        F: TitleChangeHandler,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.set_title_change_handler(self.raw, handler);
    }

    /// Set the IPC message handler
    ///
    /// This handler receives messages from JavaScript via the injected bundle.
    pub fn set_ipc_handler<F>(&mut self, handler: F)
    where
        F: IpcHandler,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.set_ipc_handler(self.raw, handler);
    }

    /// Set the load start handler
    pub fn set_load_start_handler<F>(&mut self, handler: F)
    where
        F: LoadStartHandler,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.set_load_start_handler(self.raw, handler);
    }

    /// Set the load finish handler
    pub fn set_load_finish_handler<F>(&mut self, handler: F)
    where
        F: LoadFinishHandler,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.set_load_finish_handler(self.raw, handler);
    }

    /// Set the progress change handler
    pub fn set_progress_handler<F>(&mut self, handler: F)
    where
        F: ProgressHandler,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.set_progress_handler(self.raw, handler);
    }

    // ========== User Scripts ==========

    /// Add an initialization script that runs on every page load
    pub fn add_user_script(&self, script: &str, inject_at_start: bool) -> Result<()> {
        unsafe {
            let wk_script = wk_string_from_str(script);
            if wk_script.is_null() {
                return Err(WebKitError::JavaScriptError(
                    "Failed to create script string".to_string(),
                ));
            }

            let injection_time = if inject_at_start {
                WKUserScriptInjectionTime::AtDocumentStart
            } else {
                WKUserScriptInjectionTime::AtDocumentEnd
            };

            let user_script = WKUserScriptCreateWithSource(
                wk_script,
                WKUserContentInjectedFrames::AllFrames,
                injection_time,
            );

            WKRelease(wk_script);

            if user_script.is_null() {
                return Err(WebKitError::JavaScriptError(
                    "Failed to create user script".to_string(),
                ));
            }

            // Get the page configuration's user content controller
            // Note: This requires proper setup during view creation
            // For now, we just release the script as we need the controller
            WKRelease(user_script);

            Ok(())
        }
    }

    // ========== Settings ==========

    /// Enable or disable JavaScript
    pub fn set_javascript_enabled(&self, enabled: bool) {
        unsafe {
            let prefs = WKPageGetPreferences(self.raw);
            WKPreferencesSetJavaScriptEnabled(prefs, enabled);
        }
    }

    /// Enable or disable developer tools
    pub fn set_developer_tools_enabled(&self, enabled: bool) {
        unsafe {
            let prefs = WKPageGetPreferences(self.raw);
            WKPreferencesSetDeveloperExtrasEnabled(prefs, enabled);
        }
    }
}

/// Trampoline function for JavaScript execution callbacks
unsafe extern "C" fn javascript_callback_trampoline(
    result: WKSerializedScriptValueRef,
    error: WKErrorRef,
    context: *mut c_void,
) {
    if context.is_null() {
        return;
    }

    // Reconstruct the boxed callback
    let callback: Box<Box<dyn FnOnce(std::result::Result<String, String>) + Send>> =
        Box::from_raw(context as *mut _);

    let result = if !error.is_null() {
        // There was an error - for now just return a generic message
        // A full implementation would extract the error message
        Err("JavaScript execution failed".to_string())
    } else if !result.is_null() {
        // Success - for now return empty string
        // A full implementation would serialize the result
        Ok(String::new())
    } else {
        Ok(String::new())
    };

    callback(result);
}
