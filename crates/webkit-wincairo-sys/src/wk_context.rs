//! WebKit context and configuration functions
//!
//! WKContext is the main entry point for WebKit. It manages shared resources
//! like the process pool, cookie storage, and caches.

use super::wk_types::*;

extern "C" {
    // ========== WKContext ==========

    /// Create a new WebKit context
    pub fn WKContextCreate() -> WKContextRef;

    /// Create a context with a specific configuration path
    pub fn WKContextCreateWithConfiguration(
        configuration_path: WKStringRef,
    ) -> WKContextRef;

    /// Get the default/shared context
    pub fn WKContextGetSharedProcessPool() -> WKContextRef;

    /// Set the injected bundle path for the context
    pub fn WKContextSetInjectedBundlePath(
        context: WKContextRef,
        bundle_path: WKStringRef,
    );

    /// Set additional plugins directory
    pub fn WKContextSetAdditionalPluginsDirectory(
        context: WKContextRef,
        plugins_directory: WKStringRef,
    );

    // ========== WKPageGroup ==========

    /// Create a page group with an identifier
    pub fn WKPageGroupCreateWithIdentifier(identifier: WKStringRef) -> WKPageGroupRef;

    /// Get the page group's identifier
    pub fn WKPageGroupCopyIdentifier(page_group: WKPageGroupRef) -> WKStringRef;

    // ========== WKPageConfiguration ==========

    /// Create a default page configuration
    pub fn WKPageConfigurationCreate() -> WKPageConfigurationRef;

    /// Set the context for a page configuration
    pub fn WKPageConfigurationSetContext(
        configuration: WKPageConfigurationRef,
        context: WKContextRef,
    );

    /// Get the context from a page configuration
    pub fn WKPageConfigurationGetContext(
        configuration: WKPageConfigurationRef,
    ) -> WKContextRef;

    /// Set the page group for a page configuration
    pub fn WKPageConfigurationSetPageGroup(
        configuration: WKPageConfigurationRef,
        page_group: WKPageGroupRef,
    );

    /// Get the page group from a page configuration
    pub fn WKPageConfigurationGetPageGroup(
        configuration: WKPageConfigurationRef,
    ) -> WKPageGroupRef;

    /// Set the user content controller
    pub fn WKPageConfigurationSetUserContentController(
        configuration: WKPageConfigurationRef,
        user_content_controller: WKUserContentControllerRef,
    );

    /// Get the user content controller
    pub fn WKPageConfigurationGetUserContentController(
        configuration: WKPageConfigurationRef,
    ) -> WKUserContentControllerRef;

    // ========== WKUserContentController ==========

    /// Create a user content controller
    pub fn WKUserContentControllerCreate() -> WKUserContentControllerRef;

    /// Add a user script to the controller
    pub fn WKUserContentControllerAddUserScript(
        controller: WKUserContentControllerRef,
        user_script: WKUserScriptRef,
    );

    /// Remove all user scripts
    pub fn WKUserContentControllerRemoveAllUserScripts(
        controller: WKUserContentControllerRef,
    );

    // ========== WKUserScript ==========

    /// Create a user script
    pub fn WKUserScriptCreateWithSource(
        source: WKStringRef,
        injected_frames: WKUserContentInjectedFrames,
        injection_time: WKUserScriptInjectionTime,
    ) -> WKUserScriptRef;

    /// Get the source of a user script
    pub fn WKUserScriptCopySource(user_script: WKUserScriptRef) -> WKStringRef;

    /// Get the injection time of a user script
    pub fn WKUserScriptGetInjectionTime(
        user_script: WKUserScriptRef,
    ) -> WKUserScriptInjectionTime;

    /// Get the injected frames setting of a user script
    pub fn WKUserScriptGetInjectedFrames(
        user_script: WKUserScriptRef,
    ) -> WKUserContentInjectedFrames;
}
