//! WebKit page operations
//!
//! WebKitPage represents a web page and provides methods for navigation,
//! JavaScript execution, and content manipulation.
//!
//! Note: Function signatures verified against WebKit2.dll WinCairo exports.

use crate::callbacks::{
    CallbackState, IpcHandler, LoadFinishHandler, LoadStartHandler, NavigationHandler,
    ProgressHandler, TitleChangeHandler,
};
use crate::error::{Result, WebKitError};
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use webkit_wincairo_sys::*;

type JsEvalCallback = Box<dyn FnOnce(std::result::Result<String, String>) + Send + 'static>;
/// A WebKit page
///
/// The page represents a single web document and provides methods for
/// navigation, JavaScript execution, and event handling.
pub struct WebKitPage {
    raw: WKPageRef,
    /// Callback state (kept alive for the lifetime of the page)
    #[allow(dead_code)]
    callbacks: Rc<RefCell<CallbackState>>,
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
            callbacks: Rc::new(RefCell::new(CallbackState::new())),
        }
    }

    /// Get the raw page reference
    pub(crate) fn raw(&self) -> WKPageRef {
        self.raw
    }

    // ========== Navigation ==========

    /// Load a URL
    pub fn load_url(&self, url: &str) -> Result<()> {
        log::debug!("Loading URL: {}", url);
        if url.is_empty() {
            return Err(WebKitError::InvalidUrl("URL cannot be empty".to_string()));
        }

        unsafe {
            let wk_url = wk_url_from_str(url);
            if wk_url.is_null() {
                log::error!("Failed to create WKURLRef for: {}", url);
                return Err(WebKitError::InvalidUrl(url.to_string()));
            }
            WKPageLoadURL(self.raw, wk_url);
            WKRelease(wk_url);
            Ok(())
        }
    }

    /// Load HTML content with an optional base URL
    pub fn load_html(&self, html: &str, base_url: Option<&str>) -> Result<()> {
        log::debug!("Loading {} bytes of HTML", html.len());
        unsafe {
            let wk_html = wk_string_from_str(html);
            if wk_html.is_null() {
                log::error!("Failed to create WKStringRef for HTML content");
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

    /// Get the current URL (active/displayed URL)
    pub fn url(&self) -> Option<String> {
        unsafe {
            let wk_url = WKPageCopyActiveURL(self.raw);
            let result = wk_url_to_string(wk_url);
            if !wk_url.is_null() {
                WKRelease(wk_url);
            }
            result
        }
    }

    /// Get the committed URL
    pub fn committed_url(&self) -> Option<String> {
        unsafe {
            let wk_url = WKPageCopyCommittedURL(self.raw);
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
    /// Note: Determined by progress value (loading when 0 < progress < 1)
    pub fn is_loading(&self) -> bool {
        let progress = self.progress();
        progress > 0.0 && progress < 1.0
    }

    /// Check if the page is closed
    pub fn is_closed(&self) -> bool {
        unsafe { WKPageIsClosed(self.raw) }
    }

    /// Get the user agent
    pub fn user_agent(&self) -> Option<String> {
        unsafe {
            let wk_ua = WKPageCopyUserAgent(self.raw);
            let result = wk_string_to_string(wk_ua);
            if !wk_ua.is_null() {
                WKRelease(wk_ua);
            }
            result
        }
    }

    /// Set a custom user agent
    pub fn set_user_agent(&self, user_agent: &str) {
        unsafe {
            let wk_ua = wk_string_from_str(user_agent);
            WKPageSetCustomUserAgent(self.raw, wk_ua);
            WKRelease(wk_ua);
        }
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
            let cb: JsEvalCallback = Box::new(callback);
            let callback_box: Box<JsEvalCallback> = Box::new(cb);
            let context = Box::into_raw(callback_box) as *mut c_void;

            WKPageEvaluateJavaScriptInMainFrame(
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

    // ========== Repaint ==========

    /// Force a repaint of the page
    /// This triggers WebKit to redraw the content
    pub fn force_repaint(&self) {
        unsafe {
            WKPageForceRepaint(self.raw, std::ptr::null_mut(), force_repaint_callback);
        }
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
            WKPageExecuteCommand(self.raw, wk_command, std::ptr::null_mut());
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

    /// Begin printing (requires print info setup)
    pub fn begin_printing(&self) {
        unsafe {
            let main_frame = WKPageGetMainFrame(self.raw);
            if !main_frame.is_null() {
                // Note: A full implementation would create proper print info
                WKPageBeginPrinting(self.raw, main_frame, std::ptr::null_mut());
            }
        }
    }

    /// End printing
    pub fn end_printing(&self) {
        unsafe {
            WKPageEndPrinting(self.raw);
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
        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.set_navigation_handler(self.raw, handler);
    }

    /// Set the title change handler
    pub fn set_title_change_handler<F>(&mut self, handler: F)
    where
        F: TitleChangeHandler,
    {
        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.set_title_change_handler(self.raw, handler);
    }

    /// Set the IPC message handler
    ///
    /// This handler receives messages from JavaScript via the injected bundle.
    pub fn set_ipc_handler<F>(&mut self, handler: F)
    where
        F: IpcHandler,
    {
        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.set_ipc_handler(self.raw, handler);
    }

    /// Set the load start handler
    pub fn set_load_start_handler<F>(&mut self, handler: F)
    where
        F: LoadStartHandler,
    {
        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.set_load_start_handler(self.raw, handler);
    }

    /// Set the load finish handler
    pub fn set_load_finish_handler<F>(&mut self, handler: F)
    where
        F: LoadFinishHandler,
    {
        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.set_load_finish_handler(self.raw, handler);
    }

    /// Set the progress change handler
    pub fn set_progress_handler<F>(&mut self, handler: F)
    where
        F: ProgressHandler,
    {
        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.set_progress_handler(self.raw, handler);
    }

    // ========== User Scripts & Message Handlers ==========

    /// Add a script message handler that can receive messages from JavaScript
    ///
    /// JavaScript can call `window.webkit.messageHandlers.<name>.postMessage(body)`
    /// and the callback will be invoked with the message body.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the message handler (used in JS as messageHandlers.<name>)
    /// - `callback`: Function called when a message is received
    /// - `context`: User data pointer passed to the callback
    ///
    /// # Safety
    ///
    /// The callback must be a valid function pointer and the context must remain valid
    /// for the lifetime of the message handler.
    pub unsafe fn add_script_message_handler(
        &self,
        name: &str,
        callback: unsafe extern "C" fn(WKScriptMessageRef, WKCompletionListenerRef, *const c_void),
        context: *const c_void,
    ) -> Result<()> {
        unsafe {
            let wk_name = wk_string_from_str(name);
            if wk_name.is_null() {
                return Err(WebKitError::JavaScriptError(
                    "Failed to create handler name string".to_string(),
                ));
            }

            // Get the page's user content controller via configuration
            let config = WKPageCopyPageConfiguration(self.raw);
            if config.is_null() {
                WKRelease(wk_name);
                return Err(WebKitError::JavaScriptError(
                    "Failed to get page configuration".to_string(),
                ));
            }

            let controller = WKPageConfigurationGetUserContentController(config);
            if controller.is_null() {
                WKRelease(config);
                WKRelease(wk_name);
                return Err(WebKitError::JavaScriptError(
                    "Failed to get user content controller".to_string(),
                ));
            }

            WKUserContentControllerAddScriptMessageHandler(
                controller,
                wk_name,
                Some(callback),
                context,
            );

            WKRelease(config);
            WKRelease(wk_name);
            Ok(())
        }
    }

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

            // Create user script with: source, injection_time, main_frame_only
            let user_script = WKUserScriptCreateWithSource(
                wk_script,
                injection_time,
                false, // inject into all frames
            );

            WKRelease(wk_script);

            if user_script.is_null() {
                return Err(WebKitError::JavaScriptError(
                    "Failed to create user script".to_string(),
                ));
            }

            // Get the page's user content controller via configuration
            let config = WKPageCopyPageConfiguration(self.raw);
            if !config.is_null() {
                let controller = WKPageConfigurationGetUserContentController(config);
                if !controller.is_null() {
                    WKUserContentControllerAddUserScript(controller, user_script);
                } else {
                    log::warn!("User content controller is NULL - script not added");
                }
                WKRelease(config);
            } else {
                log::warn!("Page configuration is NULL - script not added");
            }

            WKRelease(user_script);
            Ok(())
        }
    }

    // ========== Settings ==========

    /// Enable or disable JavaScript
    /// Note: Preferences must be set on the page configuration before page creation
    /// for full effect. This attempts to modify via the page group.
    pub fn set_javascript_enabled(&self, enabled: bool) {
        unsafe {
            let page_group = WKPageGetPageGroup(self.raw);
            if !page_group.is_null() {
                let prefs = WKPageGroupGetPreferences(page_group);
                if !prefs.is_null() {
                    WKPreferencesSetJavaScriptEnabled(prefs, enabled);
                }
            }
        }
    }

    /// Enable or disable developer tools
    pub fn set_developer_tools_enabled(&self, enabled: bool) {
        unsafe {
            let page_group = WKPageGetPageGroup(self.raw);
            if !page_group.is_null() {
                let prefs = WKPageGroupGetPreferences(page_group);
                if !prefs.is_null() {
                    WKPreferencesSetDeveloperExtrasEnabled(prefs, enabled);
                }
            }
        }
    }

    /// Enable or disable remote inspection (Web Inspector)
    pub fn set_allows_remote_inspection(&self, allow: bool) {
        unsafe {
            WKPageSetAllowsRemoteInspection(self.raw, allow);
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
    let callback: Box<JsEvalCallback> = Box::from_raw(context as *mut _);
    let cb: JsEvalCallback = *callback;

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

    cb(result);
}

/// Callback for WKPageForceRepaint - fire and forget
extern "C" fn force_repaint_callback(_context: *mut c_void, _error: WKTypeRef) {}
