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
const CHROME_HTML: &str = include_str!("../ui/chrome.html");
const SHELF_HTML: &str = include_str!("../ui/shelf.html");
const ABOUT_HTML: &str = include_str!("../ui/about.html");

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
        // Chrome view covers the sidebar area (left column, full height)
        // The chrome.html contains the sidebar UI
        let chrome_bounds = Bounds::new(0, 0, self.sidebar_width, self.window_height);
        let content_bounds = self.calculate_content_bounds();
        let shelf_bounds = self.calculate_shelf_bounds();

        // Create Chrome view (tabs, address bar, toolbar)
        let chrome_id = engine
            .create_view(parent, chrome_bounds)
            .map_err(|e| format!("Failed to create Chrome view: {}", e))?;
        self.views.insert(ViewType::Chrome, chrome_id);
        self.engine_view_types.insert(chrome_id, ViewType::Chrome);
        debug!(?chrome_id, "Chrome view created");

        // Create Content view (web page rendering)
        let content_id = engine
            .create_view(parent, content_bounds)
            .map_err(|e| format!("Failed to create Content view: {}", e))?;
        self.views.insert(ViewType::Content, content_id);
        self.engine_view_types.insert(content_id, ViewType::Content);
        debug!(?content_id, "Content view created");

        // Create Shelf view (command palette, hidden by default)
        let shelf_id = engine
            .create_view(parent, shelf_bounds)
            .map_err(|e| format!("Failed to create Shelf view: {}", e))?;
        self.views.insert(ViewType::Shelf, shelf_id);
        self.engine_view_types.insert(shelf_id, ViewType::Shelf);
        debug!(?shelf_id, "Shelf view created");
        
        info!(chrome = ?chrome_id, content = ?content_id, shelf = ?shelf_id, "All views created");

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
            debug!(bytes = CHROME_HTML.len(), "Chrome HTML loaded");
        }

        // Load Shelf UI
        if let Some(&shelf_id) = self.views.get(&ViewType::Shelf) {
            engine
                .load_html(shelf_id, SHELF_HTML)
                .map_err(|e| format!("Failed to load Shelf HTML: {}", e))?;
            debug!(bytes = SHELF_HTML.len(), "Shelf HTML loaded");
        }

        // Load About page in content view
        if let Some(&content_id) = self.views.get(&ViewType::Content) {
            engine
                .load_html(content_id, ABOUT_HTML)
                .map_err(|e| format!("Failed to load About HTML: {}", e))?;
            debug!(bytes = ABOUT_HTML.len(), "About page loaded");
        }

        info!("Initial content loaded into all views");
        Ok(())
    }

    /// Calculate bounds for the content view.
    fn calculate_content_bounds(&self) -> Bounds {
        let x = if self.sidebar_open {
            self.sidebar_width as i32
        } else {
            0
        };
        let y = 0; // Content starts at top (no toolbar in native mode)
        let width = if self.sidebar_open {
            self.window_width.saturating_sub(self.sidebar_width)
        } else {
            self.window_width
        };
        let height = self
            .window_height
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

        // Update Chrome view (sidebar area - left column, full height)
        if let Some(&chrome_id) = self.views.get(&ViewType::Chrome) {
            let width = if self.sidebar_open { self.sidebar_width } else { 0 };
            let bounds = Bounds::new(0, 0, width, self.window_height);
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
        // Parse the JSON payload
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&msg.payload);
        let json = match parsed {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, payload_len = msg.payload.len(), "Failed to parse IPC message JSON");
                return;
            }
        };

        // Extract the command
        let cmd = match json.get("cmd").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => {
                trace!("IPC message missing 'cmd' field (likely data payload)");
                return;
            }
        };

        // Log at trace level for high-frequency commands, debug for others
        match cmd {
            // High-frequency, log at trace only
            "log" | "sync_tabs" | "sync_workspaces" | "sync_downloads" => {
                trace!(?view_type, cmd, "IPC");
            }
            // Low-frequency, log at debug
            _ => {
                debug!(?view_type, cmd, "IPC command");
            }
        }

        // Handle the command
        match cmd {
            "navigate" => {
                if let Some(url) = json.get("url").and_then(|v| v.as_str()) {
                    info!(url, "Navigation started");
                    self.navigate(url);
                }
            }
            "go_back" => {
                debug!("History: back");
                // TODO: Implement navigation history
            }
            "go_forward" => {
                debug!("History: forward");
                // TODO: Implement navigation history
            }
            "reload" => {
                debug!("Page reload");
                // TODO: Implement reload
            }
            "expand_chrome" => {
                trace!("Layout: chrome expanded");
                self.expand_chrome();
            }
            "collapse_chrome" => {
                trace!("Layout: chrome collapsed");
                self.collapse_chrome();
            }
            "expand_shelf" | "open_command_palette" => {
                debug!("Shelf opened");
                self.expand_shelf();
            }
            "collapse_shelf" | "close_command_palette" => {
                trace!("Shelf closed");
                self.collapse_shelf();
            }
            "toggle_sidebar" => {
                debug!(open = !self.sidebar_open, "Sidebar toggled");
                self.toggle_sidebar();
            }
            "chrome_ready" => {
                info!("Chrome UI initialized and ready");
                // TODO: Send initial state to Chrome UI
            }
            "log" => {
                // JavaScript console log forwarding
                let level = json.get("level").and_then(|v| v.as_str()).unwrap_or("info");
                let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("");
                match level {
                    "error" => error!(source = "js", "{}", message),
                    "warn" => warn!(source = "js", "{}", message),
                    "debug" => trace!(source = "js", "{}", message), // Downgrade JS debug to trace
                    _ => trace!(source = "js", "{}", message), // JS info → trace (very frequent)
                }
            }
            _ => {
                trace!(cmd, "Unhandled IPC command (may be hybrid-only)");
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

/// Print startup banner with build info.
fn print_startup_banner() {
    info!("=== HiWave Browser ===");
    info!(
        version = env!("CARGO_PKG_VERSION"),
        engine = "RustKit",
        mode = "native-win32",
        "Startup"
    );
    info!(
        target_os = std::env::consts::OS,
        target_arch = std::env::consts::ARCH,
        "Platform"
    );
    
    // Get DPI scale if available
    #[cfg(windows)]
    {
        use windows::Win32::UI::HiDpi::GetDpiForSystem;
        let dpi = unsafe { GetDpiForSystem() };
        let scale = dpi as f64 / 96.0;
        info!(dpi, scale, "Display");
    }
}

/// Install panic hook for structured crash logging.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Log the panic with tracing
        let location = panic_info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
        let message = panic_info.payload().downcast_ref::<&str>().map(|s| *s)
            .or_else(|| panic_info.payload().downcast_ref::<String>().map(|s| s.as_str()));
        
        error!(
            location = location.as_deref(),
            message = message,
            "PANIC - Browser crashed"
        );
        
        // Call the default hook for normal panic behavior
        default_hook(panic_info);
    }));
}

/// Entry point for native Win32 mode.
///
/// This function is called from main.rs when the `native-win32` feature is enabled.
pub fn run_native() -> Result<(), String> {
    install_panic_hook();
    print_startup_banner();

    let mut browser = NativeBrowser::new()?;
    browser.init()?;
    browser.run();

    info!("HiWave shutdown complete");
    Ok(())
}

