//! Windows platform implementation
//!
//! Supports two WebView engines:
//! - WebView2 (default): Chromium-based, less aggressive with popups
//! - WinCairo WebKit (optional): WebKit-based, more aggressive popup filtering
//!
//! The native menu is optional but provides a consistent user experience.

use super::{
    is_definitely_popup, is_legitimate_navigation, PlatformCapabilities, PlatformError,
    PlatformManager, PlatformResult, WebViewEngine,
};
use muda::Menu;
use std::path::{Path, PathBuf};
use std::process::Command;
use tao::window::Window;
use tracing::debug;

/// Windows platform manager
pub struct WindowsPlatform {
    capabilities: PlatformCapabilities,
}

impl WindowsPlatform {
    pub fn new() -> Self {
        let default_download_dir = dirs::download_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Downloads")
        });

        // Detect which WebView engine to use based on feature flags
        #[cfg(feature = "wincairo")]
        let webview_engine = WebViewEngine::WinCairoWebKit;

        #[cfg(not(feature = "wincairo"))]
        let webview_engine = WebViewEngine::WebView2;

        Self {
            capabilities: PlatformCapabilities {
                native_menu_required_for_clipboard: false,
                supports_file_selection_in_folder: true,
                webview_engine,
                default_download_dir,
                platform_name: "Windows",
            },
        }
    }

    /// Check if using WinCairo WebKit engine
    #[allow(dead_code)]
    pub fn is_using_wincairo(&self) -> bool {
        matches!(
            self.capabilities.webview_engine,
            WebViewEngine::WinCairoWebKit
        )
    }

    /// Check if using WebView2 (Chromium) engine
    #[allow(dead_code)]
    pub fn is_using_webview2(&self) -> bool {
        matches!(self.capabilities.webview_engine, WebViewEngine::WebView2)
    }
}

impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformManager for WindowsPlatform {
    fn initialize_menu(&self, window: &Window, menu: &Menu) -> PlatformResult<()> {
        // Initialize menu for the window's HWND
        // Note: On Windows, we need the raw window handle
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            unsafe {
                menu.init_for_hwnd(window.hwnd() as _).map_err(|e| {
                    PlatformError::MenuInitFailed(format!("HWND init failed: {}", e))
                })?;
            }
        }

        debug!("Windows menu initialized successfully");
        Ok(())
    }

    fn open_external(&self, url: &str) -> PlatformResult<()> {
        debug!("Opening external URL: {}", url);

        // Use 'cmd /C start' to open URLs on Windows
        // The empty "" after start is the window title (required for URLs with special chars)
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| PlatformError::OpenExternalFailed(format!("{}: {}", url, e)))?;

        Ok(())
    }

    fn open_file(&self, path: &Path) -> PlatformResult<()> {
        if !path.exists() {
            return Err(PlatformError::FileNotFound(path.to_path_buf()));
        }

        debug!("Opening file: {:?}", path);

        // Convert path to string, handling potential Unicode issues
        let path_str = path.to_str().ok_or_else(|| {
            PlatformError::OpenFileFailed("Path contains invalid Unicode".to_string())
        })?;

        Command::new("cmd")
            .args(["/C", "start", "", path_str])
            .spawn()
            .map_err(|e| PlatformError::OpenFileFailed(format!("{:?}: {}", path, e)))?;

        Ok(())
    }

    fn show_in_folder(&self, path: &Path) -> PlatformResult<()> {
        if !path.exists() {
            return Err(PlatformError::FileNotFound(path.to_path_buf()));
        }

        debug!("Showing in Explorer: {:?}", path);

        // Convert path to string
        let path_str = path.to_str().ok_or_else(|| {
            PlatformError::ShowInFolderFailed("Path contains invalid Unicode".to_string())
        })?;

        // Use explorer /select to open folder and select the file
        Command::new("explorer")
            .args(["/select,", path_str])
            .spawn()
            .map_err(|e| PlatformError::ShowInFolderFailed(format!("{:?}: {}", path, e)))?;

        Ok(())
    }

    fn capabilities(&self) -> &PlatformCapabilities {
        &self.capabilities
    }

    fn should_open_as_tab(&self, url: &str, _initiating_url: Option<&str>) -> bool {
        // Quick rejection for obvious popups (applies to both engines)
        if is_definitely_popup(url) {
            debug!("Blocking popup (definitely): {}", url);
            return false;
        }

        // Block about: and data: URLs
        if url == "about:blank" || url == "about:srcdoc" || url.starts_with("data:") {
            debug!("Blocking about/data URL: {}", url);
            return false;
        }

        // Block javascript: URLs
        if url.starts_with("javascript:") {
            debug!("Blocking javascript URL: {}", url);
            return false;
        }

        // WinCairo WebKit uses more aggressive filtering (same as macOS WebKit)
        if self.capabilities.webview_engine.is_webkit_based() {
            // Extended ad/tracking patterns for WebKit
            let blocked_patterns = [
                "googlesyndication.com",
                "doubleclick.net",
                "googleadservices.com",
                "facebook.com/tr",
                "connect.facebook.net",
                "criteo.com",
                "tapad.com",
                "lijit.com",
                "/beacon",
                "/pixel",
                "/track",
                "/sdk/",
            ];

            let url_lower = url.to_lowercase();
            for pattern in blocked_patterns {
                if url_lower.contains(pattern) {
                    debug!(
                        "Blocking URL matching pattern '{}' (WebKit): {}",
                        pattern, url
                    );
                    return false;
                }
            }
        } else {
            // WebView2 uses lighter filtering
            let blocked_patterns = [
                "googlesyndication.com",
                "doubleclick.net",
                "googleadservices.com",
                "facebook.com/tr",
            ];

            let url_lower = url.to_lowercase();
            for pattern in blocked_patterns {
                if url_lower.contains(pattern) {
                    debug!("Blocking URL matching pattern '{}': {}", pattern, url);
                    return false;
                }
            }
        }

        // Allow if it looks legitimate
        if is_legitimate_navigation(url) {
            return true;
        }

        // Default behavior depends on engine
        // WebKit: more conservative (block by default)
        // WebView2: more permissive (allow by default)
        !self.capabilities.webview_engine.is_webkit_based()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_capabilities() {
        let platform = WindowsPlatform::new();
        let caps = platform.capabilities();

        assert!(!caps.native_menu_required_for_clipboard);
        assert!(caps.supports_file_selection_in_folder);
        assert_eq!(caps.platform_name, "Windows");

        // Engine depends on feature flag
        #[cfg(feature = "wincairo")]
        assert_eq!(caps.webview_engine, WebViewEngine::WinCairoWebKit);

        #[cfg(not(feature = "wincairo"))]
        assert_eq!(caps.webview_engine, WebViewEngine::WebView2);
    }

    #[test]
    fn test_should_open_as_tab_blocks_popups() {
        let platform = WindowsPlatform::new();

        // Should block on all engines
        assert!(!platform.should_open_as_tab("about:blank", None));
        assert!(!platform.should_open_as_tab("javascript:void(0)", None));
        assert!(!platform.should_open_as_tab("https://doubleclick.net/ad", None));
    }

    #[test]
    fn test_should_open_as_tab_allows_legitimate() {
        let platform = WindowsPlatform::new();

        // Should allow legitimate URLs
        assert!(platform.should_open_as_tab("https://github.com/pureflow", None));
        assert!(platform.should_open_as_tab("https://example.com/page", None));
    }

    #[cfg(not(feature = "wincairo"))]
    #[test]
    fn test_webview2_less_aggressive() {
        let platform = WindowsPlatform::new();
        let caps = platform.capabilities();

        // WebView2 should not be marked as aggressive
        assert!(!caps.webview_engine.is_aggressive_popup_opener());
        assert!(caps.webview_engine.is_chromium_based());
        assert!(!caps.webview_engine.is_webkit_based());
    }

    #[cfg(feature = "wincairo")]
    #[test]
    fn test_wincairo_aggressive() {
        let platform = WindowsPlatform::new();
        let caps = platform.capabilities();

        // WinCairo WebKit should be marked as aggressive
        assert!(caps.webview_engine.is_aggressive_popup_opener());
        assert!(caps.webview_engine.is_webkit_based());
        assert!(!caps.webview_engine.is_chromium_based());
    }

    #[test]
    fn test_engine_detection_helpers() {
        let platform = WindowsPlatform::new();

        #[cfg(feature = "wincairo")]
        {
            assert!(platform.is_using_wincairo());
            assert!(!platform.is_using_webview2());
        }

        #[cfg(not(feature = "wincairo"))]
        {
            assert!(!platform.is_using_wincairo());
            assert!(platform.is_using_webview2());
        }
    }
}
