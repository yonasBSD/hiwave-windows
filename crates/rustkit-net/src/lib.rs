//! # RustKit Net
//!
//! HTTP networking, request interception, and download management for the RustKit browser engine.
//!
//! ## Design Goals
//!
//! 1. **Async HTTP**: Non-blocking network requests
//! 2. **Request interception**: Filter/modify/block requests
//! 3. **Download management**: Progress, pause, resume, cancel
//! 4. **fetch() API**: JavaScript-compatible fetch interface

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use mime::Mime;
use reqwest::Client;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, trace, warn};
use url::Url;

pub mod download;
pub mod intercept;

pub use download::{Download, DownloadEvent, DownloadId, DownloadManager, DownloadState};
pub use intercept::{InterceptAction, InterceptHandler, RequestInterceptor};

/// Errors that can occur in networking.
#[derive(Error, Debug)]
pub enum NetError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Request cancelled")]
    Cancelled,

    #[error("Request blocked")]
    Blocked,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

/// Unique identifier for a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

impl RequestId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    pub id: RequestId,
    pub url: Url,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
    pub timeout: Option<Duration>,
    pub credentials: CredentialsMode,
    pub referrer: Option<Url>,
}

impl Request {
    /// Create a GET request.
    pub fn get(url: Url) -> Self {
        Self {
            id: RequestId::new(),
            url,
            method: Method::GET,
            headers: HeaderMap::new(),
            body: None,
            timeout: Some(Duration::from_secs(30)),
            credentials: CredentialsMode::SameOrigin,
            referrer: None,
        }
    }

    /// Create a POST request.
    pub fn post(url: Url, body: Bytes) -> Self {
        Self {
            id: RequestId::new(),
            url,
            method: Method::POST,
            headers: HeaderMap::new(),
            body: Some(body),
            timeout: Some(Duration::from_secs(30)),
            credentials: CredentialsMode::SameOrigin,
            referrer: None,
        }
    }

    /// Add a header.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }

    /// Set timeout.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set referrer.
    pub fn referrer(mut self, referrer: Url) -> Self {
        self.referrer = Some(referrer);
        self
    }
}

/// Credentials mode for requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CredentialsMode {
    /// Never send cookies.
    Omit,
    /// Send cookies only for same-origin requests.
    #[default]
    SameOrigin,
    /// Always send cookies.
    Include,
}

/// HTTP response.
#[derive(Debug)]
pub struct Response {
    pub request_id: RequestId,
    pub url: Url,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub content_type: Option<Mime>,
    pub content_length: Option<u64>,
    body: ResponseBody,
}

/// Response body variants.
#[derive(Debug)]
#[allow(dead_code)]
enum ResponseBody {
    /// Full body already loaded.
    Full(Bytes),
    /// Streaming body.
    Stream(mpsc::Receiver<Result<Bytes, NetError>>),
    /// Empty.
    Empty,
}

impl Response {
    /// Check if request was successful (2xx).
    pub fn ok(&self) -> bool {
        self.status.is_success()
    }

    /// Get the body as bytes.
    pub async fn bytes(self) -> Result<Bytes, NetError> {
        match self.body {
            ResponseBody::Full(b) => Ok(b),
            ResponseBody::Stream(mut rx) => {
                let mut chunks = Vec::new();
                while let Some(chunk) = rx.recv().await {
                    chunks.push(chunk?);
                }
                Ok(chunks.into_iter().flatten().collect())
            }
            ResponseBody::Empty => Ok(Bytes::new()),
        }
    }

    /// Get the body as text.
    pub async fn text(self) -> Result<String, NetError> {
        let bytes = self.bytes().await?;
        String::from_utf8(bytes.to_vec()).map_err(|e| NetError::RequestFailed(e.to_string()))
    }

    /// Get the body as JSON.
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, NetError> {
        let bytes = self.bytes().await?;
        serde_json::from_slice(&bytes).map_err(|e| NetError::RequestFailed(e.to_string()))
    }

    /// Get a suggested filename from Content-Disposition or URL.
    pub fn suggested_filename(&self) -> Option<String> {
        // Try Content-Disposition header
        if let Some(cd) = self.headers.get("content-disposition") {
            if let Ok(cd_str) = cd.to_str() {
                if let Some(start) = cd_str.find("filename=") {
                    let start = start + 9;
                    let filename = &cd_str[start..];
                    let filename = filename.trim_matches('"').trim_matches('\'');
                    if let Some(end) = filename.find(';') {
                        return Some(filename[..end].to_string());
                    }
                    return Some(filename.to_string());
                }
            }
        }

        // Fall back to URL path
        self.url
            .path_segments()
            .and_then(|mut segments| segments.next_back())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }
}

/// Resource loader configuration.
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// User agent string.
    pub user_agent: String,
    /// Accept-Language header.
    pub accept_language: String,
    /// Default timeout.
    pub default_timeout: Duration,
    /// Maximum redirects.
    pub max_redirects: usize,
    /// Enable cookies.
    pub cookies_enabled: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            user_agent: "RustKit/1.0".to_string(),
            accept_language: "en-US,en;q=0.9".to_string(),
            default_timeout: Duration::from_secs(30),
            max_redirects: 10,
            cookies_enabled: true,
        }
    }
}

/// Resource loader for fetching URLs.
pub struct ResourceLoader {
    client: Client,
    config: LoaderConfig,
    interceptor: Option<Arc<RwLock<RequestInterceptor>>>,
    download_manager: Arc<DownloadManager>,
}

impl ResourceLoader {
    /// Create a new resource loader.
    pub fn new(config: LoaderConfig) -> Result<Self, NetError> {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.default_timeout)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects))
            .cookie_store(config.cookies_enabled)
            .build()
            .map_err(|e| NetError::RequestFailed(e.to_string()))?;

        info!("ResourceLoader initialized");

        Ok(Self {
            client,
            config,
            interceptor: None,
            download_manager: Arc::new(DownloadManager::new()),
        })
    }

    /// Set the request interceptor.
    pub fn set_interceptor(&mut self, interceptor: RequestInterceptor) {
        self.interceptor = Some(Arc::new(RwLock::new(interceptor)));
    }

    /// Get the download manager.
    pub fn download_manager(&self) -> Arc<DownloadManager> {
        Arc::clone(&self.download_manager)
    }

    /// Fetch a URL.
    pub async fn fetch(&self, request: Request) -> Result<Response, NetError> {
        debug!(url = %request.url, method = %request.method, "Fetching resource");

        // Apply interception
        if let Some(interceptor) = &self.interceptor {
            let action = interceptor.read().await.intercept(&request).await;
            match action {
                InterceptAction::Allow => {}
                InterceptAction::Block => {
                    warn!(url = %request.url, "Request blocked by interceptor");
                    return Err(NetError::Blocked);
                }
                InterceptAction::Redirect(new_url) => {
                    debug!(url = %request.url, new_url = %new_url, "Request redirected");
                    let mut new_request = request.clone();
                    new_request.url = new_url;
                    return Box::pin(self.fetch(new_request)).await;
                }
                InterceptAction::Modify(modified) => {
                    return Box::pin(self.fetch(*modified)).await;
                }
            }
        }

        // Build reqwest request
        let mut req_builder = self
            .client
            .request(request.method.clone(), request.url.clone());

        // Add headers
        for (name, value) in request.headers.iter() {
            req_builder = req_builder.header(name, value);
        }

        // Add Accept-Language
        req_builder = req_builder.header("Accept-Language", &self.config.accept_language);

        // Add referrer
        if let Some(ref referrer) = request.referrer {
            req_builder = req_builder.header("Referer", referrer.as_str());
        }

        // Add body
        if let Some(body) = request.body {
            req_builder = req_builder.body(body);
        }

        // Set timeout
        if let Some(timeout) = request.timeout {
            req_builder = req_builder.timeout(timeout);
        }

        // Execute request
        let response = req_builder.send().await?;

        let status = response.status();
        let headers = response.headers().clone();
        let url = response.url().clone();

        // Parse content type
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<Mime>().ok());

        // Get content length
        let content_length = headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        // Read body
        let body = response.bytes().await?;

        trace!(
            url = %url,
            status = %status,
            content_type = ?content_type,
            content_length = ?content_length,
            body_len = body.len(),
            "Response received"
        );

        Ok(Response {
            request_id: request.id,
            url,
            status,
            headers,
            content_type,
            content_length,
            body: ResponseBody::Full(body),
        })
    }

    /// Start a download.
    pub async fn start_download(
        &self,
        url: Url,
        destination: PathBuf,
    ) -> Result<DownloadId, NetError> {
        let request = Request::get(url);
        self.download_manager
            .start(request, destination, self.client.clone())
            .await
    }
}

/// Fetch API for JavaScript compatibility.
pub struct FetchApi {
    loader: Arc<ResourceLoader>,
}

impl FetchApi {
    /// Create a new fetch API.
    pub fn new(loader: Arc<ResourceLoader>) -> Self {
        Self { loader }
    }

    /// Fetch with options similar to JavaScript fetch().
    pub async fn fetch(&self, url: &str, options: FetchOptions) -> Result<Response, NetError> {
        let url = Url::parse(url).map_err(|e| NetError::InvalidUrl(e.to_string()))?;

        let mut request = match options.method.as_deref() {
            Some("POST") => Request::post(url, options.body.unwrap_or_default()),
            Some("PUT") => {
                let mut req = Request::get(url);
                req.method = Method::PUT;
                req.body = options.body;
                req
            }
            Some("DELETE") => {
                let mut req = Request::get(url);
                req.method = Method::DELETE;
                req
            }
            _ => Request::get(url),
        };

        // Add headers
        for (name, value) in options.headers {
            if let (Ok(n), Ok(v)) = (
                HeaderName::try_from(name.as_str()),
                HeaderValue::try_from(value.as_str()),
            ) {
                request.headers.insert(n, v);
            }
        }

        // Set credentials
        request.credentials = match options.credentials.as_deref() {
            Some("omit") => CredentialsMode::Omit,
            Some("include") => CredentialsMode::Include,
            _ => CredentialsMode::SameOrigin,
        };

        self.loader.fetch(request).await
    }
}

/// Options for fetch API.
#[derive(Debug, Default)]
pub struct FetchOptions {
    pub method: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<Bytes>,
    pub credentials: Option<String>,
    pub mode: Option<String>,
    pub cache: Option<String>,
    pub redirect: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let url = Url::parse("https://example.com").unwrap();
        let request = Request::get(url.clone())
            .header(
                HeaderName::from_static("accept"),
                HeaderValue::from_static("application/json"),
            )
            .timeout(Duration::from_secs(10));

        assert_eq!(request.url, url);
        assert_eq!(request.method, Method::GET);
        assert!(request.headers.contains_key("accept"));
        assert_eq!(request.timeout, Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_request_id_uniqueness() {
        let id1 = RequestId::new();
        let id2 = RequestId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_credentials_mode_default() {
        assert_eq!(CredentialsMode::default(), CredentialsMode::SameOrigin);
    }

    #[test]
    fn test_loader_config_default() {
        let config = LoaderConfig::default();
        assert_eq!(config.user_agent, "RustKit/1.0");
        assert!(config.cookies_enabled);
    }
}
