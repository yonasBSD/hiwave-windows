//! Native Win32 Entry Point for HiWave
//!
//! This module provides a pure Win32 entry point for HiWave that uses RustKit
//! for all rendering, completely bypassing wry/tao.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  Win32 Main Window (ViewHost)           │
//! ├─────────────────────────────────────────┤
//! │  RustKit Chrome View (tabs, toolbar)    │
//! ├─────────────────────────────────────────┤
//! │  RustKit Content View (web pages)       │
//! ├─────────────────────────────────────────┤
//! │  RustKit Shelf View (command palette)   │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Feature Flag
//!
//! This module is only compiled when the `native-win32` feature is enabled:
//!
//! ```bash
//! cargo build --features native-win32
//! ```

use rustkit_engine::{Engine, EngineBuilder, EngineViewId, IpcMessage};
use rustkit_viewhost::{Bounds, MainWindowConfig, ViewEvent, ViewHost};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};
use windows::Win32::Foundation::HWND;

use crate::state::AppState;

// Chrome UI constants
const CHROME_HEIGHT_DEFAULT: u32 = 104;
const CHROME_HEIGHT_EXPANDED: u32 = 460;
const SHELF_HEIGHT_DEFAULT: u32 = 0;
const SHELF_HEIGHT_EXPANDED: u32 = 280;
const SIDEBAR_WIDTH: u32 = 220;

/// HTML content embedded at compile time
const CHROME_HTML: &str = include_str!("ui/chrome.html");
const SHELF_HTML: &str = include_str!("ui/shelf.html");
const ABOUT_HTML: &str = include_str!("ui/about.html");

/// View types in the browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewType {
    Chrome,
    Content,
    Shelf,
}

/// State for the native Win32 browser
pub struct NativeBrowser {
    /// ViewHost manages all Win32 windows
    viewhost: ViewHost,
    /// RustKit engine for rendering
    engine: RefCell<Engine>,
    /// Map of view types to engine view IDs
    views: HashMap<ViewType, EngineViewId>,
    /// Reverse map: engine view ID to view type (for IPC routing)
    engine_view_types: HashMap<EngineViewId, ViewType>,
    /// Application state
    #[allow(dead_code)]
    app_state: Arc<AppState>,
    /// Current window dimensions
    window_width: u32,
    window_height: u32,
    /// Layout state
    chrome_height: u32,
    shelf_height: u32,
    sidebar_width: u32,
    sidebar_open: bool,
}

impl NativeBrowser {
    /// Create a new native browser instance.
    pub fn new() -> Result<Self, String> {
        info!("Initializing native Win32 browser");

        // Create the engine
        let engine = EngineBuilder::new()
            .user_agent("HiWave/1.0 RustKit/1.0")
            .javascript_enabled(true)
            .cookies_enabled(true)
            .build()
            .map_err(|e| format!("Failed to create engine: {}", e))?;

        // Initialize app state
        let app_state = Arc::new(
            AppState::with_defaults()
                .map_err(|e| format!("Failed to initialize app state: {}", e))?,
        );

        Ok(Self {
            viewhost: ViewHost::new(),
            engine: RefCell::new(engine),
            views: HashMap::new(),
            engine_view_types: HashMap::new(),
            app_state,
            window_width: 1280,
            window_height: 800,
            chrome_height: CHROME_HEIGHT_DEFAULT,
            shelf_height: SHELF_HEIGHT_DEFAULT,
            sidebar_width: SIDEBAR_WIDTH,
            sidebar_open: true,
        })
    }

    /// Initialize the browser window and views.
    pub fn init(&mut self) -> Result<(), String> {
        // Create the main window
        let config = MainWindowConfig::new("HiWave").with_size(1280, 800);
        let hwnd = self
            .viewhost
            .create_main_window(config)
            .map_err(|e| format!("Failed to create window: {}", e))?;

        // Set up event callback
        let _callback = Arc::new(move |event: ViewEvent| {
            // Events are handled in the message loop
            debug!(?event, "View event received");
        });

        // Create the three views
        self.create_views(hwnd)?;

        // Load initial content
        self.load_initial_content()?;

        info!("Native browser initialized");
        Ok(())
    }

    /// Create Chrome, Content, and Shelf views.
    fn create_views(&mut self, parent: HWND) -> Result<(), String> {
        let mut engine = self.engine.borrow_mut();

        // Calculate initial bounds
        let chrome_bounds = Bounds::new(0, 0, self.window_width, self.chrome_height);
        let content_bounds = self.calculate_content_bounds();
        let shelf_bounds = self.calculate_shelf_bounds();

        // Create Chrome view (tabs, address bar, toolbar)
        let chrome_id = engine
            .create_view(parent, chrome_bounds)
            .map_err(|e| format!("Failed to create Chrome view: {}", e))?;
        self.views.insert(ViewType::Chrome, chrome_id);
        self.engine_view_types.insert(chrome_id, ViewType::Chrome);
        info!(?chrome_id, "Chrome view created");

        // Create Content view (web page rendering)
        let content_id = engine
            .create_view(parent, content_bounds)
            .map_err(|e| format!("Failed to create Content view: {}", e))?;
        self.views.insert(ViewType::Content, content_id);
        self.engine_view_types.insert(content_id, ViewType::Content);
        info!(?content_id, "Content view created");

        // Create Shelf view (command palette, hidden by default)
        let shelf_id = engine
            .create_view(parent, shelf_bounds)
            .map_err(|e| format!("Failed to create Shelf view: {}", e))?;
        self.views.insert(ViewType::Shelf, shelf_id);
        self.engine_view_types.insert(shelf_id, ViewType::Shelf);
        info!(?shelf_id, "Shelf view created");

        Ok(())
    }

    /// Load initial HTML content into views.
    fn load_initial_content(&self) -> Result<(), String> {
        let mut engine = self.engine.borrow_mut();

        // Load Chrome UI
        if let Some(&chrome_id) = self.views.get(&ViewType::Chrome) {
            engine
                .load_html(chrome_id, CHROME_HTML)
                .map_err(|e| format!("Failed to load Chrome HTML: {}", e))?;
            info!("Chrome UI loaded");
        }

        // Load Shelf UI
        if let Some(&shelf_id) = self.views.get(&ViewType::Shelf) {
            engine
                .load_html(shelf_id, SHELF_HTML)
                .map_err(|e| format!("Failed to load Shelf HTML: {}", e))?;
            info!("Shelf UI loaded");
        }

        // Load About page in content view
        if let Some(&content_id) = self.views.get(&ViewType::Content) {
            engine
                .load_html(content_id, ABOUT_HTML)
                .map_err(|e| format!("Failed to load About HTML: {}", e))?;
            info!("About page loaded in content view");
        }

        Ok(())
    }

    /// Calculate bounds for the content view.
    fn calculate_content_bounds(&self) -> Bounds {
        let x = if self.sidebar_open {
            self.sidebar_width as i32
        } else {
            0
        };
        let y = self.chrome_height as i32;
        let width = if self.sidebar_open {
            self.window_width.saturating_sub(self.sidebar_width)
        } else {
            self.window_width
        };
        let height = self
            .window_height
            .saturating_sub(self.chrome_height)
            .saturating_sub(self.shelf_height);

        Bounds::new(x, y, width, height)
    }

    /// Calculate bounds for the shelf view.
    fn calculate_shelf_bounds(&self) -> Bounds {
        let x = if self.sidebar_open {
            self.sidebar_width as i32
        } else {
            0
        };
        let y = (self.window_height - self.shelf_height) as i32;
        let width = if self.sidebar_open {
            self.window_width.saturating_sub(self.sidebar_width)
        } else {
            self.window_width
        };

        Bounds::new(x, y, width, self.shelf_height)
    }

    /// Update layout after window resize or layout change.
    fn update_layout(&self) {
        let mut engine = self.engine.borrow_mut();

        // Update Chrome view (full width, fixed height at top)
        if let Some(&chrome_id) = self.views.get(&ViewType::Chrome) {
            let bounds = Bounds::new(0, 0, self.window_width, self.chrome_height);
            if let Err(e) = engine.resize_view(chrome_id, bounds) {
                warn!(error = %e, "Failed to resize Chrome view");
            }
        }

        // Update Content view
        if let Some(&content_id) = self.views.get(&ViewType::Content) {
            let bounds = self.calculate_content_bounds();
            if let Err(e) = engine.resize_view(content_id, bounds) {
                warn!(error = %e, "Failed to resize Content view");
            }
        }

        // Update Shelf view
        if let Some(&shelf_id) = self.views.get(&ViewType::Shelf) {
            let bounds = self.calculate_shelf_bounds();
            if let Err(e) = engine.resize_view(shelf_id, bounds) {
                warn!(error = %e, "Failed to resize Shelf view");
            }
        }
    }

    /// Handle window resize event.
    pub fn on_resize(&mut self, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;
        self.update_layout();
    }

    /// Expand the Chrome UI (for overlays).
    pub fn expand_chrome(&mut self) {
        self.chrome_height = CHROME_HEIGHT_EXPANDED;
        self.update_layout();
    }

    /// Collapse the Chrome UI to default height.
    pub fn collapse_chrome(&mut self) {
        self.chrome_height = CHROME_HEIGHT_DEFAULT;
        self.update_layout();
    }

    /// Expand the Shelf (command palette).
    pub fn expand_shelf(&mut self) {
        self.shelf_height = SHELF_HEIGHT_EXPANDED;
        self.update_layout();
    }

    /// Collapse the Shelf.
    pub fn collapse_shelf(&mut self) {
        self.shelf_height = SHELF_HEIGHT_DEFAULT;
        self.update_layout();
    }

    /// Toggle sidebar visibility.
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_open = !self.sidebar_open;
        self.update_layout();
    }

    /// Navigate to a URL.
    pub fn navigate(&self, url: &str) {
        if let Some(&content_id) = self.views.get(&ViewType::Content) {
            match url::Url::parse(url) {
                Ok(parsed) => {
                    let mut engine = self.engine.borrow_mut();
                    // Create a runtime for the async operation
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to create runtime");

                    rt.block_on(async {
                        if let Err(e) = engine.load_url(content_id, parsed).await {
                            error!(error = %e, url, "Failed to navigate");
                        }
                    });
                }
                Err(e) => {
                    warn!(error = %e, url, "Invalid URL");
                }
            }
        }
    }

    /// Execute JavaScript in a view.
    pub fn execute_script(&self, view_type: ViewType, script: &str) -> Option<String> {
        if let Some(&view_id) = self.views.get(&view_type) {
            let mut engine = self.engine.borrow_mut();
            match engine.execute_script(view_id, script) {
                Ok(result) => Some(result),
                Err(e) => {
                    debug!(error = %e, "Script execution failed");
                    None
                }
            }
        } else {
            None
        }
    }

    /// Run the browser's main message loop with IPC processing.
    pub fn run(&mut self) {
        info!("Starting native browser message loop");

        loop {
            // Process Win32 messages (non-blocking)
            if !self.viewhost.pump_messages() {
                // WM_QUIT received, exit the loop
                break;
            }

            // Render all views
            self.engine.borrow_mut().render_all_views();

            // Process any IPC messages from views
            self.process_ipc_messages();

            // Small sleep to prevent busy-waiting (target ~60fps)
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        info!("Browser message loop ended");
    }

    /// Process IPC messages from all views.
    fn process_ipc_messages(&mut self) {
        let messages = self.engine.borrow().drain_ipc_messages();

        for (view_id, ipc_msg) in messages {
            let view_type = self.engine_view_types.get(&view_id).copied();
            self.handle_ipc_message(view_type, ipc_msg);
        }
    }

    /// Handle a single IPC message.
    fn handle_ipc_message(&mut self, view_type: Option<ViewType>, msg: IpcMessage) {
        trace!(?view_type, payload = %msg.payload, "IPC message received");

        // Parse the JSON payload
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&msg.payload);
        let json = match parsed {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "Failed to parse IPC message JSON");
                return;
            }
        };

        // Extract the command
        let cmd = match json.get("cmd").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => {
                warn!("IPC message missing 'cmd' field");
                return;
            }
        };

        // Handle the command
        match cmd {
            "navigate" => {
                if let Some(url) = json.get("url").and_then(|v| v.as_str()) {
                    info!(url, "Navigate requested");
                    self.navigate(url);
                }
            }
            "go_back" => {
                debug!("Go back requested");
                // TODO: Implement navigation history
            }
            "go_forward" => {
                debug!("Go forward requested");
                // TODO: Implement navigation history
            }
            "reload" => {
                debug!("Reload requested");
                // TODO: Implement reload
            }
            "expand_chrome" => {
                self.expand_chrome();
            }
            "collapse_chrome" => {
                self.collapse_chrome();
            }
            "expand_shelf" | "open_command_palette" => {
                self.expand_shelf();
            }
            "collapse_shelf" | "close_command_palette" => {
                self.collapse_shelf();
            }
            "toggle_sidebar" => {
                self.toggle_sidebar();
            }
            "chrome_ready" => {
                info!("Chrome UI ready");
                // TODO: Send initial state to Chrome UI
            }
            "log" => {
                let level = json.get("level").and_then(|v| v.as_str()).unwrap_or("info");
                let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("");
                match level {
                    "error" => error!(source = "js", "{}", message),
                    "warn" => warn!(source = "js", "{}", message),
                    "debug" => debug!(source = "js", "{}", message),
                    _ => info!(source = "js", "{}", message),
                }
            }
            _ => {
                debug!(cmd, "Unhandled IPC command");
            }
        }
    }
}

impl Drop for NativeBrowser {
    fn drop(&mut self) {
        info!("Shutting down native browser");

        // Destroy all views
        let mut engine = self.engine.borrow_mut();
        for (&view_type, &view_id) in &self.views {
            if let Err(e) = engine.destroy_view(view_id) {
                warn!(?view_type, error = %e, "Failed to destroy view");
            }
        }
    }
}

/// Entry point for native Win32 mode.
///
/// This function is called from main.rs when the `native-win32` feature is enabled.
pub fn run_native() -> Result<(), String> {
    info!("Starting HiWave in native Win32 mode");

    let mut browser = NativeBrowser::new()?;
    browser.init()?;
    browser.run();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_calculation() {
        // Test that bounds calculations don't panic
        let browser = NativeBrowser {
            viewhost: ViewHost::new(),
            engine: RefCell::new(
                EngineBuilder::new()
                    .build()
                    .expect("Failed to create test engine"),
            ),
            views: HashMap::new(),
            engine_view_types: HashMap::new(),
            app_state: Arc::new(AppState::new()),
            window_width: 1280,
            window_height: 800,
            chrome_height: CHROME_HEIGHT_DEFAULT,
            shelf_height: SHELF_HEIGHT_DEFAULT,
            sidebar_width: SIDEBAR_WIDTH,
            sidebar_open: true,
        };

        let content_bounds = browser.calculate_content_bounds();
        assert_eq!(content_bounds.x, SIDEBAR_WIDTH as i32);
        assert_eq!(content_bounds.y, CHROME_HEIGHT_DEFAULT as i32);
        assert!(content_bounds.width > 0);
        assert!(content_bounds.height > 0);

        let shelf_bounds = browser.calculate_shelf_bounds();
        assert_eq!(shelf_bounds.x, SIDEBAR_WIDTH as i32);
        assert_eq!(shelf_bounds.height, 0); // Shelf is collapsed by default
    }
}
