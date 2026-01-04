//! macOS ViewHost implementation using Cocoa/AppKit.
//!
//! This module provides a native macOS window hosting layer using NSWindow and NSView.
//! It translates Cocoa events to platform-agnostic RustKit events.
//!
//! ## Architecture
//!
//! - `NSWindow` for top-level windows
//! - `NSView` subclass for each view
//! - `CAMetalLayer` for wgpu surface integration
//! - Event responder chain for input handling

#![cfg(target_os = "macos")]

use crate::{Bounds, EventCallback, MainWindowConfig, ViewEvent, ViewHostError, ViewId};
use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSEvent,
    NSEventMask, NSEventType, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopRunInMode, kCFRunLoopDefaultMode};
use core_graphics::display::CGDisplay;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{msg_send, sel, sel_impl};
use rustkit_core::{
    FocusEvent, FocusEventType, InputEvent, KeyCode, KeyEvent, KeyEventType, KeyboardState,
    Modifiers, MouseButton, MouseEvent, MouseEventType, MouseState, Point,
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, RwLock};
use tracing::{debug, error, info, trace, warn};

/// View state for macOS (stores NSView reference).
struct MacOSViewState {
    id: ViewId,
    view: id,       // NSView
    bounds: Bounds,
    visible: bool,
    focused: bool,
    scale_factor: f64,
    keyboard_state: KeyboardState,
    mouse_state: MouseState,
}

/// macOS ViewHost implementation.
pub struct MacOSViewHost {
    views: RwLock<HashMap<ViewId, Arc<Mutex<MacOSViewState>>>>,
    main_window: RwLock<Option<id>>,
    event_callback: RwLock<Option<EventCallback>>,
    app_initialized: bool,
}

impl MacOSViewHost {
    /// Create a new macOS ViewHost.
    pub fn new() -> Self {
        info!("Initializing macOS ViewHost");
        
        Self {
            views: RwLock::new(HashMap::new()),
            main_window: RwLock::new(None),
            event_callback: RwLock::new(None),
            app_initialized: false,
        }
    }

    /// Initialize the NSApplication if not already done.
    fn ensure_app_initialized(&mut self) {
        if self.app_initialized {
            return;
        }

        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let app = NSApp();
            app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);
            self.app_initialized = true;
            debug!("NSApplication initialized");
        }
    }

    /// Create a top-level main window.
    pub fn create_main_window(&mut self, config: MainWindowConfig) -> Result<id, ViewHostError> {
        self.ensure_app_initialized();

        info!(?config, "Creating macOS main window");

        unsafe {
            let _pool = NSAutoreleasePool::new(nil);

            // Calculate frame
            let (x, y) = if config.centered {
                let screen = CGDisplay::main();
                let screen_width = screen.pixels_wide() as f64;
                let screen_height = screen.pixels_high() as f64;
                (
                    (screen_width - config.width as f64) / 2.0,
                    (screen_height - config.height as f64) / 2.0,
                )
            } else {
                (100.0, 100.0) // Default position
            };

            let frame = NSRect::new(
                NSPoint::new(x, y),
                NSSize::new(config.width as f64, config.height as f64),
            );

            // Window style
            let mut style = NSWindowStyleMask::NSTitledWindowMask
                | NSWindowStyleMask::NSClosableWindowMask
                | NSWindowStyleMask::NSMiniaturizableWindowMask;

            if config.resizable {
                style |= NSWindowStyleMask::NSResizableWindowMask;
            }

            // Create window
            let window: id = msg_send![
                NSWindow::alloc(nil),
                initWithContentRect:frame
                styleMask:style
                backing:NSBackingStoreType::NSBackingStoreBuffered
                defer:NO
            ];

            if window == nil {
                error!("Failed to create NSWindow");
                return Err(ViewHostError::WindowCreation("NSWindow creation failed".into()));
            }

            // Set title
            let title = NSString::alloc(nil).init_str(&config.title);
            let _: () = msg_send![window, setTitle: title];

            // Make key and order front
            let _: () = msg_send![window, makeKeyAndOrderFront: nil];

            // Activate the app
            let app = NSApp();
            let _: () = msg_send![app, activateIgnoringOtherApps: YES];

            // Store the main window
            *self.main_window.write().unwrap() = Some(window);

            info!("macOS main window created successfully");
            Ok(window)
        }
    }

    /// Create a child view in the given parent.
    pub fn create_view(&self, parent: id, bounds: Bounds) -> Result<ViewId, ViewHostError> {
        let view_id = ViewId::new();
        debug!(?view_id, ?bounds, "Creating macOS view");

        unsafe {
            let _pool = NSAutoreleasePool::new(nil);

            let frame = NSRect::new(
                NSPoint::new(bounds.x as f64, bounds.y as f64),
                NSSize::new(bounds.width as f64, bounds.height as f64),
            );

            // Create NSView
            let view: id = msg_send![class!(NSView), alloc];
            let view: id = msg_send![view, initWithFrame: frame];

            if view == nil {
                error!(?view_id, "Failed to create NSView");
                return Err(ViewHostError::WindowCreation("NSView creation failed".into()));
            }

            // Add as subview
            let content_view: id = msg_send![parent, contentView];
            if content_view != nil {
                let _: () = msg_send![content_view, addSubview: view];
            }

            // Get scale factor
            let window: id = msg_send![view, window];
            let scale_factor: f64 = if window != nil {
                msg_send![window, backingScaleFactor]
            } else {
                1.0
            };

            // Create state
            let state = MacOSViewState {
                id: view_id,
                view,
                bounds,
                visible: true,
                focused: false,
                scale_factor,
                keyboard_state: KeyboardState::default(),
                mouse_state: MouseState::default(),
            };

            self.views
                .write()
                .unwrap()
                .insert(view_id, Arc::new(Mutex::new(state)));

            debug!(?view_id, scale_factor, "macOS view created");
            Ok(view_id)
        }
    }

    /// Resize a view to new bounds.
    pub fn resize_view(&self, view_id: ViewId, bounds: Bounds) -> Result<(), ViewHostError> {
        let views = self.views.read().unwrap();
        let state = views
            .get(&view_id)
            .ok_or(ViewHostError::ViewNotFound(view_id))?;

        let mut state = state.lock().unwrap();
        state.bounds = bounds;

        unsafe {
            let frame = NSRect::new(
                NSPoint::new(bounds.x as f64, bounds.y as f64),
                NSSize::new(bounds.width as f64, bounds.height as f64),
            );
            let _: () = msg_send![state.view, setFrame: frame];
        }

        trace!(?view_id, ?bounds, "View resized");
        Ok(())
    }

    /// Destroy a view.
    pub fn destroy_view(&self, view_id: ViewId) -> Result<(), ViewHostError> {
        let state = self
            .views
            .write()
            .unwrap()
            .remove(&view_id)
            .ok_or(ViewHostError::ViewNotFound(view_id))?;

        let state = state.lock().unwrap();
        unsafe {
            let _: () = msg_send![state.view, removeFromSuperview];
        }

        debug!(?view_id, "macOS view destroyed");
        Ok(())
    }

    /// Set the event callback.
    pub fn set_event_callback(&self, callback: EventCallback) {
        *self.event_callback.write().unwrap() = Some(callback);
    }

    /// Pump events from the macOS event loop (non-blocking).
    ///
    /// Returns `true` if the app should continue, `false` if it should quit.
    pub fn pump_messages(&self) -> bool {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);

            // Process all pending events
            loop {
                let event: id = msg_send![
                    NSApp(),
                    nextEventMatchingMask: NSEventMask::NSAnyEventMask.bits()
                    untilDate: nil
                    inMode: NSString::alloc(nil).init_str("kCFRunLoopDefaultMode")
                    dequeue: YES
                ];

                if event == nil {
                    break;
                }

                let event_type: NSEventType = msg_send![event, type];

                // Check for app termination
                if event_type == NSEventType::NSApplicationDefined {
                    // Application is terminating
                    return false;
                }

                // Dispatch the event
                let _: () = msg_send![NSApp(), sendEvent: event];
            }
        }

        true
    }

    /// Get the scale factor for a view.
    pub fn get_scale_factor(&self, view_id: ViewId) -> f64 {
        self.views
            .read()
            .unwrap()
            .get(&view_id)
            .map(|s| s.lock().unwrap().scale_factor)
            .unwrap_or(1.0)
    }

    /// Emit an event via the callback.
    fn emit_event(&self, event: ViewEvent) {
        if let Some(ref callback) = *self.event_callback.read().unwrap() {
            callback(event);
        }
    }
}

impl Default for MacOSViewHost {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert NSEvent key code to RustKit KeyCode.
fn translate_key_code(key_code: u16) -> KeyCode {
    // macOS virtual key codes
    match key_code {
        0x00 => KeyCode::KeyA,
        0x0B => KeyCode::KeyB,
        0x08 => KeyCode::KeyC,
        0x02 => KeyCode::KeyD,
        0x0E => KeyCode::KeyE,
        0x03 => KeyCode::KeyF,
        0x05 => KeyCode::KeyG,
        0x04 => KeyCode::KeyH,
        0x22 => KeyCode::KeyI,
        0x26 => KeyCode::KeyJ,
        0x28 => KeyCode::KeyK,
        0x25 => KeyCode::KeyL,
        0x2E => KeyCode::KeyM,
        0x2D => KeyCode::KeyN,
        0x1F => KeyCode::KeyO,
        0x23 => KeyCode::KeyP,
        0x0C => KeyCode::KeyQ,
        0x0F => KeyCode::KeyR,
        0x01 => KeyCode::KeyS,
        0x11 => KeyCode::KeyT,
        0x20 => KeyCode::KeyU,
        0x09 => KeyCode::KeyV,
        0x0D => KeyCode::KeyW,
        0x07 => KeyCode::KeyX,
        0x10 => KeyCode::KeyY,
        0x06 => KeyCode::KeyZ,
        0x12 => KeyCode::Digit1,
        0x13 => KeyCode::Digit2,
        0x14 => KeyCode::Digit3,
        0x15 => KeyCode::Digit4,
        0x17 => KeyCode::Digit5,
        0x16 => KeyCode::Digit6,
        0x1A => KeyCode::Digit7,
        0x1C => KeyCode::Digit8,
        0x19 => KeyCode::Digit9,
        0x1D => KeyCode::Digit0,
        0x24 => KeyCode::Enter,
        0x35 => KeyCode::Escape,
        0x33 => KeyCode::Backspace,
        0x30 => KeyCode::Tab,
        0x31 => KeyCode::Space,
        0x7E => KeyCode::ArrowUp,
        0x7D => KeyCode::ArrowDown,
        0x7B => KeyCode::ArrowLeft,
        0x7C => KeyCode::ArrowRight,
        _ => KeyCode::Unknown,
    }
}

/// Convert NSEvent modifier flags to RustKit Modifiers.
fn translate_modifiers(flags: u64) -> Modifiers {
    let mut mods = Modifiers::empty();
    
    // macOS modifier flag constants
    const NSEventModifierFlagShift: u64 = 1 << 17;
    const NSEventModifierFlagControl: u64 = 1 << 18;
    const NSEventModifierFlagOption: u64 = 1 << 19;
    const NSEventModifierFlagCommand: u64 = 1 << 20;

    if flags & NSEventModifierFlagShift != 0 {
        mods |= Modifiers::SHIFT;
    }
    if flags & NSEventModifierFlagControl != 0 {
        mods |= Modifiers::CTRL;
    }
    if flags & NSEventModifierFlagOption != 0 {
        mods |= Modifiers::ALT;
    }
    if flags & NSEventModifierFlagCommand != 0 {
        mods |= Modifiers::META;
    }

    mods
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_key_code() {
        assert_eq!(translate_key_code(0x00), KeyCode::KeyA);
        assert_eq!(translate_key_code(0x24), KeyCode::Enter);
        assert_eq!(translate_key_code(0x35), KeyCode::Escape);
    }

    #[test]
    fn test_translate_modifiers() {
        let mods = translate_modifiers(1 << 17); // Shift
        assert!(mods.contains(Modifiers::SHIFT));
        
        let mods = translate_modifiers(1 << 20); // Command
        assert!(mods.contains(Modifiers::META));
    }
}

