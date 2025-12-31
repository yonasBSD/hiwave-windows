//! Error types for webkit-wincairo

use thiserror::Error;

/// Errors that can occur when using WebKit
#[derive(Error, Debug)]
pub enum WebKitError {
    /// Failed to create a WebKit context
    #[error("Failed to create WebKit context")]
    ContextCreationFailed,

    /// Failed to create a WebKit view
    #[error("Failed to create WebKit view")]
    ViewCreationFailed,

    /// Failed to create a WebKit page
    #[error("Failed to create WebKit page")]
    PageCreationFailed,

    /// Failed to load a URL
    #[error("Failed to load URL: {0}")]
    LoadUrlFailed(String),

    /// Failed to load HTML content
    #[error("Failed to load HTML content")]
    LoadHtmlFailed,

    /// Failed to execute JavaScript
    #[error("Failed to execute JavaScript: {0}")]
    JavaScriptError(String),

    /// JavaScript execution timed out
    #[error("JavaScript execution timed out")]
    JavaScriptTimeout,

    /// Invalid URL provided
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Invalid HTML provided
    #[error("Invalid HTML content")]
    InvalidHtml,

    /// WebKit is not initialized
    #[error("WebKit is not initialized")]
    NotInitialized,

    /// Operation failed due to null pointer
    #[error("Null pointer encountered")]
    NullPointer,

    /// String conversion error
    #[error("String conversion error: {0}")]
    StringConversionError(String),

    /// Callback registration failed
    #[error("Failed to register callback: {0}")]
    CallbackRegistrationFailed(String),

    /// View bounds are invalid
    #[error("Invalid view bounds: {0}")]
    InvalidBounds(String),

    /// Window handle is invalid
    #[error("Invalid window handle")]
    InvalidWindowHandle,

    /// Internal WebKit error
    #[error("Internal WebKit error: {0}")]
    Internal(String),
}

/// Result type alias for webkit-wincairo operations
pub type Result<T> = std::result::Result<T, WebKitError>;
