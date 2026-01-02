//! # RustKit Bindings
//!
//! JavaScript-to-DOM bindings for the RustKit browser engine.
//!
//! ## Design Goals
//!
//! 1. **Web compatibility**: Match browser API behavior
//! 2. **Type safety**: Safe conversion between JS and Rust types
//! 3. **Performance**: Minimize overhead at the boundary
//! 4. **Extensibility**: Easy to add new APIs

use rustkit_dom::{Document, Node, NodeId};
use rustkit_js::{JsError, JsRuntime, JsValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use tracing::{debug, trace};
use url::Url;

/// Errors that can occur in bindings.
#[derive(Error, Debug)]
pub enum BindingError {
    #[error("DOM error: {0}")]
    DomError(String),

    #[error("JS error: {0}")]
    JsError(#[from] JsError),

    #[error("Type error: expected {expected}, got {got}")]
    TypeError { expected: String, got: String },

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

/// Unique identifier for an event listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(u64);

impl ListenerId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// An event listener registration.
#[derive(Debug)]
pub struct EventListener {
    pub id: ListenerId,
    pub node_id: NodeId,
    pub event_type: String,
    pub callback: String, // JS code to execute
    pub capture: bool,
}

/// Location object (window.location).
#[derive(Debug, Clone)]
pub struct Location {
    pub href: String,
    pub protocol: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub origin: String,
}

impl Location {
    /// Create a Location from a URL.
    pub fn from_url(url: &Url) -> Self {
        Self {
            href: url.to_string(),
            protocol: format!("{}:", url.scheme()),
            host: url
                .host_str()
                .map(|h| {
                    if let Some(port) = url.port() {
                        format!("{}:{}", h, port)
                    } else {
                        h.to_string()
                    }
                })
                .unwrap_or_default(),
            hostname: url.host_str().unwrap_or("").to_string(),
            port: url.port().map(|p| p.to_string()).unwrap_or_default(),
            pathname: url.path().to_string(),
            search: url.query().map(|q| format!("?{}", q)).unwrap_or_default(),
            hash: url
                .fragment()
                .map(|f| format!("#{}", f))
                .unwrap_or_default(),
            origin: url.origin().unicode_serialization(),
        }
    }

    /// Create a Location from a string.
    pub fn from_string(href: &str) -> Result<Self, BindingError> {
        let url = Url::parse(href).map_err(|e| BindingError::InvalidArgument(e.to_string()))?;
        Ok(Self::from_url(&url))
    }
}

impl Default for Location {
    fn default() -> Self {
        Self {
            href: "about:blank".to_string(),
            protocol: "about:".to_string(),
            host: String::new(),
            hostname: String::new(),
            port: String::new(),
            pathname: "blank".to_string(),
            search: String::new(),
            hash: String::new(),
            origin: "null".to_string(),
        }
    }
}

/// Window object state.
pub struct WindowState {
    pub location: Location,
    pub document: Option<Rc<Document>>,
    pub name: String,
    pub inner_width: f64,
    pub inner_height: f64,
    pub outer_width: f64,
    pub outer_height: f64,
    pub device_pixel_ratio: f64,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            location: Location::default(),
            document: None,
            name: String::new(),
            inner_width: 800.0,
            inner_height: 600.0,
            outer_width: 800.0,
            outer_height: 600.0,
            device_pixel_ratio: 1.0,
        }
    }
}

/// DOM bindings context.
pub struct DomBindings {
    runtime: RefCell<JsRuntime>,
    window: RefCell<WindowState>,
    event_listeners: RefCell<Vec<EventListener>>,
    node_map: RefCell<HashMap<u64, Rc<Node>>>,
}

impl DomBindings {
    /// Create new DOM bindings.
    pub fn new(mut runtime: JsRuntime) -> Result<Self, BindingError> {
        debug!("Initializing DOM bindings");

        // Inject global objects
        Self::inject_globals(&mut runtime)?;

        Ok(Self {
            runtime: RefCell::new(runtime),
            window: RefCell::new(WindowState::default()),
            event_listeners: RefCell::new(Vec::new()),
            node_map: RefCell::new(HashMap::new()),
        })
    }

    /// Inject global JavaScript objects.
    fn inject_globals(runtime: &mut JsRuntime) -> Result<(), BindingError> {
        // Window object stub
        let window_js = r#"
            var window = {
                innerWidth: 800,
                innerHeight: 600,
                outerWidth: 800,
                outerHeight: 600,
                devicePixelRatio: 1,
                location: {
                    href: 'about:blank',
                    protocol: 'about:',
                    host: '',
                    hostname: '',
                    port: '',
                    pathname: 'blank',
                    search: '',
                    hash: '',
                    origin: 'null',
                    reload: function() {},
                    replace: function(url) { this.href = url; },
                    assign: function(url) { this.href = url; }
                },
                navigator: {
                    userAgent: 'RustKit/1.0',
                    language: 'en-US',
                    languages: ['en-US', 'en'],
                    platform: 'Win32',
                    onLine: true
                },
                history: {
                    length: 1,
                    back: function() {},
                    forward: function() {},
                    go: function(delta) {},
                    pushState: function(state, title, url) {},
                    replaceState: function(state, title, url) {}
                },
                localStorage: {
                    _data: {},
                    getItem: function(key) { return this._data[key] || null; },
                    setItem: function(key, value) { this._data[key] = String(value); },
                    removeItem: function(key) { delete this._data[key]; },
                    clear: function() { this._data = {}; },
                    get length() { return Object.keys(this._data).length; },
                    key: function(n) { return Object.keys(this._data)[n] || null; }
                },
                sessionStorage: {
                    _data: {},
                    getItem: function(key) { return this._data[key] || null; },
                    setItem: function(key, value) { this._data[key] = String(value); },
                    removeItem: function(key) { delete this._data[key]; },
                    clear: function() { this._data = {}; },
                    get length() { return Object.keys(this._data).length; },
                    key: function(n) { return Object.keys(this._data)[n] || null; }
                },
                addEventListener: function(type, callback, options) {},
                removeEventListener: function(type, callback, options) {},
                dispatchEvent: function(event) { return true; },
                requestAnimationFrame: function(callback) { return 0; },
                cancelAnimationFrame: function(id) {},
                getComputedStyle: function(element) { return {}; },
                matchMedia: function(query) {
                    return { matches: false, media: query, addEventListener: function() {} };
                },
                alert: function(msg) { console.log('[alert]', msg); },
                confirm: function(msg) { console.log('[confirm]', msg); return false; },
                prompt: function(msg, def) { console.log('[prompt]', msg); return def || null; }
            };
            
            // Alias
            var self = window;
        "#;

        runtime.evaluate_script(window_js)?;

        // Document object stub
        let document_js = r#"
            var document = {
                _elements: {},
                documentElement: null,
                head: null,
                body: null,
                title: '',
                readyState: 'loading',
                cookie: '',
                domain: '',
                referrer: '',
                URL: 'about:blank',
                
                getElementById: function(id) {
                    return this._elements[id] || null;
                },
                
                getElementsByTagName: function(tagName) {
                    return [];
                },
                
                getElementsByClassName: function(className) {
                    return [];
                },
                
                querySelector: function(selector) {
                    return null;
                },
                
                querySelectorAll: function(selector) {
                    return [];
                },
                
                createElement: function(tagName) {
                    return {
                        tagName: tagName.toUpperCase(),
                        id: '',
                        className: '',
                        textContent: '',
                        innerHTML: '',
                        style: {},
                        attributes: {},
                        children: [],
                        parentNode: null,
                        
                        getAttribute: function(name) {
                            return this.attributes[name] || null;
                        },
                        setAttribute: function(name, value) {
                            this.attributes[name] = value;
                        },
                        removeAttribute: function(name) {
                            delete this.attributes[name];
                        },
                        appendChild: function(child) {
                            this.children.push(child);
                            child.parentNode = this;
                            return child;
                        },
                        removeChild: function(child) {
                            var idx = this.children.indexOf(child);
                            if (idx >= 0) {
                                this.children.splice(idx, 1);
                                child.parentNode = null;
                            }
                            return child;
                        },
                        addEventListener: function(type, callback, options) {},
                        removeEventListener: function(type, callback, options) {}
                    };
                },
                
                createTextNode: function(text) {
                    return { nodeType: 3, textContent: text };
                },
                
                createDocumentFragment: function() {
                    return { children: [], appendChild: function(c) { this.children.push(c); return c; } };
                },
                
                addEventListener: function(type, callback, options) {},
                removeEventListener: function(type, callback, options) {},
                dispatchEvent: function(event) { return true; },
                
                write: function(html) {},
                writeln: function(html) {}
            };
            
            window.document = document;
        "#;

        runtime.evaluate_script(document_js)?;

        debug!("Global objects injected");
        Ok(())
    }

    /// Set the document.
    pub fn set_document(&self, document: Rc<Document>) -> Result<(), BindingError> {
        // Update state
        self.window.borrow_mut().document = Some(document.clone());

        // Sync to JS
        let title = document.title().unwrap_or_default();
        let mut runtime = self.runtime.borrow_mut();
        runtime.evaluate_script(&format!("document.title = {:?};", title))?;
        runtime.evaluate_script("document.readyState = 'complete';")?;

        // Index elements by ID
        document.traverse(|node| {
            if let Some(_id) = node.get_attribute("id") {
                let node_id = node.id.raw();
                self.node_map
                    .borrow_mut()
                    .insert(node_id as u64, node.clone());
            }
        });

        debug!("Document bound to JS context");
        Ok(())
    }

    /// Set the current URL.
    pub fn set_location(&self, url: &Url) -> Result<(), BindingError> {
        let location = Location::from_url(url);

        // Update state
        self.window.borrow_mut().location = location.clone();

        // Sync to JS
        let mut runtime = self.runtime.borrow_mut();
        runtime.evaluate_script(&format!(
            r#"
            window.location.href = {:?};
            window.location.protocol = {:?};
            window.location.host = {:?};
            window.location.hostname = {:?};
            window.location.port = {:?};
            window.location.pathname = {:?};
            window.location.search = {:?};
            window.location.hash = {:?};
            window.location.origin = {:?};
            document.URL = {:?};
            "#,
            location.href,
            location.protocol,
            location.host,
            location.hostname,
            location.port,
            location.pathname,
            location.search,
            location.hash,
            location.origin,
            location.href
        ))?;

        Ok(())
    }

    /// Set window dimensions.
    pub fn set_dimensions(&self, width: f64, height: f64) -> Result<(), BindingError> {
        let mut window = self.window.borrow_mut();
        window.inner_width = width;
        window.inner_height = height;
        window.outer_width = width;
        window.outer_height = height;
        drop(window);

        let mut runtime = self.runtime.borrow_mut();
        runtime.evaluate_script(&format!(
            "window.innerWidth = {}; window.innerHeight = {}; \
             window.outerWidth = {}; window.outerHeight = {};",
            width, height, width, height
        ))?;

        Ok(())
    }

    /// Evaluate a script in the bound context.
    pub fn evaluate(&self, script: &str) -> Result<JsValue, BindingError> {
        self.runtime
            .borrow_mut()
            .evaluate_script(script)
            .map_err(Into::into)
    }

    /// Add an event listener.
    pub fn add_event_listener(
        &self,
        node_id: NodeId,
        event_type: &str,
        callback: &str,
        capture: bool,
    ) -> ListenerId {
        let id = ListenerId::new();
        let listener = EventListener {
            id,
            node_id,
            event_type: event_type.to_string(),
            callback: callback.to_string(),
            capture,
        };

        self.event_listeners.borrow_mut().push(listener);
        trace!(?id, event_type, "Event listener added");
        id
    }

    /// Remove an event listener.
    pub fn remove_event_listener(&self, id: ListenerId) {
        self.event_listeners.borrow_mut().retain(|l| l.id != id);
        trace!(?id, "Event listener removed");
    }

    /// Dispatch an event.
    pub fn dispatch_event(&self, node_id: NodeId, event_type: &str) -> Result<(), BindingError> {
        let listeners: Vec<_> = self
            .event_listeners
            .borrow()
            .iter()
            .filter(|l| l.node_id == node_id && l.event_type == event_type)
            .map(|l| l.callback.clone())
            .collect();

        let mut runtime = self.runtime.borrow_mut();
        for callback in listeners {
            runtime.evaluate_script(&callback)?;
        }

        Ok(())
    }

    /// Get the current location.
    pub fn location(&self) -> Location {
        self.window.borrow().location.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_from_url() {
        let url = Url::parse("https://example.com:8080/path?query=1#hash").unwrap();
        let loc = Location::from_url(&url);

        assert_eq!(loc.href, "https://example.com:8080/path?query=1#hash");
        assert_eq!(loc.protocol, "https:");
        assert_eq!(loc.host, "example.com:8080");
        assert_eq!(loc.hostname, "example.com");
        assert_eq!(loc.port, "8080");
        assert_eq!(loc.pathname, "/path");
        assert_eq!(loc.search, "?query=1");
        assert_eq!(loc.hash, "#hash");
    }

    #[test]
    fn test_bindings_creation() {
        let runtime = JsRuntime::new().unwrap();
        let bindings = DomBindings::new(runtime).unwrap();

        // Window should exist
        let result = bindings.evaluate("typeof window").unwrap();
        assert!(matches!(result, JsValue::String(s) if s == "object"));
    }

    #[test]
    fn test_document_exists() {
        let runtime = JsRuntime::new().unwrap();
        let bindings = DomBindings::new(runtime).unwrap();

        let result = bindings.evaluate("typeof document").unwrap();
        assert!(matches!(result, JsValue::String(s) if s == "object"));
    }

    #[test]
    fn test_navigator() {
        let runtime = JsRuntime::new().unwrap();
        let bindings = DomBindings::new(runtime).unwrap();

        let result = bindings.evaluate("window.navigator.userAgent").unwrap();
        assert!(matches!(result, JsValue::String(s) if s.contains("RustKit")));
    }

    #[test]
    fn test_local_storage() {
        let runtime = JsRuntime::new().unwrap();
        let bindings = DomBindings::new(runtime).unwrap();

        bindings
            .evaluate("window.localStorage.setItem('key', 'value')")
            .unwrap();
        let result = bindings
            .evaluate("window.localStorage.getItem('key')")
            .unwrap();
        assert!(matches!(result, JsValue::String(s) if s == "value"));
    }

    #[test]
    fn test_set_dimensions() {
        let runtime = JsRuntime::new().unwrap();
        let bindings = DomBindings::new(runtime).unwrap();

        bindings.set_dimensions(1024.0, 768.0).unwrap();

        let width = bindings.evaluate("window.innerWidth").unwrap();
        assert!(matches!(width, JsValue::Number(n) if (n - 1024.0).abs() < f64::EPSILON));
    }
}
