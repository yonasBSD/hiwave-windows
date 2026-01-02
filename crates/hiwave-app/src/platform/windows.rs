//! Windows platform implementation
//!
//! Supports two WebView engines:
//! - WebView2 (default): Chromium-based, less aggressive with popups
//! - RustKit (optional): Pure Rust browser engine
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
        #[cfg(feature = "rustkit")]
        let webview_engine = WebViewEngine::RustKit;

        #[cfg(not(feature = "rustkit"))]
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

    /// Check if using RustKit engine
    #[allow(dead_code)]
    pub fn is_using_rustkit(&self) -> bool {
        matches!(self.capabilities.webview_engine, WebViewEngine::RustKit)
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

        // Standard blocking patterns
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

        // Allow if it looks legitimate
        if is_legitimate_navigation(url) {
            return true;
        }

        // WebView2: more permissive (allow by default)
        true
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
        #[cfg(feature = "rustkit")]
        assert_eq!(caps.webview_engine, WebViewEngine::RustKit);

        #[cfg(not(feature = "rustkit"))]
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

    #[test]
    fn test_engine_detection_helpers() {
        let platform = WindowsPlatform::new();

        #[cfg(feature = "rustkit")]
        {
            assert!(platform.is_using_rustkit());
            assert!(!platform.is_using_webview2());
        }

        #[cfg(not(feature = "rustkit"))]
        {
            assert!(!platform.is_using_rustkit());
            assert!(platform.is_using_webview2());
        }
    }
}
