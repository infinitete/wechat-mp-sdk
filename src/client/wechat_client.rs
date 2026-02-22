//! WeChat HTTP Client
//!
//! Provides HTTP client wrapper for WeChat API calls.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tower::Service;

use crate::error::{HttpError, WechatError};
use crate::types::{AppId, AppSecret};

pub(crate) const DEFAULT_BASE_URL: &str = "https://api.weixin.qq.com";
pub(crate) const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub(crate) const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

type MiddlewareFuture =
    Pin<Box<dyn Future<Output = Result<reqwest::Response, reqwest::Error>> + Send>>;
type MiddlewareExecutor = Arc<dyn Fn(reqwest::Request) -> MiddlewareFuture + Send + Sync>;

/// WeChat API Client
///
/// Reusable HTTP client for calling WeChat APIs.
/// Built with reqwest for async HTTP requests.
#[derive(Clone)]
pub struct WechatClient {
    http: Client,
    appid: AppId,
    secret: AppSecret,
    base_url: String,
    middleware_executor: Option<MiddlewareExecutor>,
}

impl std::fmt::Debug for WechatClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WechatClient")
            .field("appid", &self.appid)
            .field("base_url", &self.base_url)
            .field(
                "middleware_executor",
                &self.middleware_executor.as_ref().map(|_| ".."),
            )
            .finish_non_exhaustive()
    }
}

impl WechatClient {
    /// Create a new client builder
    pub fn builder() -> WechatClientBuilder {
        WechatClientBuilder::default()
    }

    /// Get the appid
    pub fn appid(&self) -> &str {
        self.appid.as_str()
    }

    /// Get the app secret
    pub(crate) fn secret(&self) -> &str {
        self.secret.as_str()
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) fn append_access_token(path: &str, access_token: &str) -> String {
        let encoded = utf8_percent_encode(access_token, NON_ALPHANUMERIC);

        if path.contains("access_token={}") {
            return path.replacen("access_token={}", &format!("access_token={encoded}"), 1);
        }

        let separator = if path.contains('?') { '&' } else { '?' };
        format!("{path}{separator}access_token={encoded}")
    }

    /// Returns the underlying [`reqwest::Client`] for raw HTTP requests.
    ///
    /// Note: requests made through this client bypass the middleware pipeline.
    /// Use [`get`](Self::get) or [`post`](Self::post) for middleware-aware requests.
    pub fn http(&self) -> &Client {
        &self.http
    }

    pub(crate) fn with_middleware_executor(mut self, executor: MiddlewareExecutor) -> Self {
        self.middleware_executor = Some(executor);
        self
    }

    pub(crate) async fn send_request(
        &self,
        request: reqwest::Request,
    ) -> Result<reqwest::Response, reqwest::Error> {
        if let Some(executor) = &self.middleware_executor {
            (executor)(request).await
        } else {
            self.http.execute(request).await
        }
    }

    async fn execute<T: DeserializeOwned>(
        &self,
        request: reqwest::Request,
    ) -> Result<T, WechatError> {
        let response = self.send_request(request).await?;

        if let Err(e) = response.error_for_status_ref() {
            return Err(e.into());
        }

        let value: serde_json::Value = response.json().await?;

        if let Some(errcode) = value.get("errcode").and_then(|v| v.as_i64()) {
            if errcode != 0 {
                let errmsg = value
                    .get("errmsg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                return Err(WechatError::Api {
                    code: errcode.try_into().unwrap_or(i32::MAX),
                    message: errmsg.to_string(),
                });
            }
        }

        serde_json::from_value(value)
            .map_err(|e| WechatError::Http(HttpError::Decode(e.to_string())))
    }

    /// Make a GET request to WeChat API
    ///
    /// # Arguments
    /// * `path` - API endpoint path (e.g., "/cgi-bin/token")
    /// * `query` - Query parameters as key-value pairs
    ///
    /// # Returns
    /// Deserialized response of type T
    ///
    /// # Errors
    /// - Returns `WechatError::Http` for non-2xx HTTP status codes or decode failures
    /// - Returns `WechatError::Api` when WeChat API returns errcode != 0
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, WechatError> {
        let url = format!("{}{}", self.base_url, path);
        let request = self.http.get(url).query(query).build()?;
        self.execute(request).await
    }

    /// Make a POST request to WeChat API
    ///
    /// # Arguments
    /// * `path` - API endpoint path (e.g., "/wxa/getwxadevinfo")
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Returns
    /// Deserialized response of type T
    ///
    /// # Errors
    /// - Returns `WechatError::Http` for non-2xx HTTP status codes or decode failures
    /// - Returns `WechatError::Api` when WeChat API returns errcode != 0
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, WechatError> {
        let url = format!("{}{}", self.base_url, path);
        let request = self.http.post(url).json(body).build()?;
        self.execute(request).await
    }
}

impl Service<reqwest::Request> for WechatClient {
    type Response = reqwest::Response;
    type Error = reqwest::Error;
    type Future = MiddlewareFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: reqwest::Request) -> Self::Future {
        let client = self.http.clone();
        Box::pin(async move { client.execute(req).await })
    }
}

/// Builder for WechatClient
///
/// # Example
///
/// ```rust
/// use wechat_mp_sdk::client::WechatClient;
/// use wechat_mp_sdk::types::{AppId, AppSecret};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let appid = AppId::new("wx1234567890abcdef")?;
///     let secret = AppSecret::new("abc1234567890abcdef")?;
///
///     let client = WechatClient::builder()
///         .appid(appid)
///         .secret(secret)
///         .build()?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Default)]
pub struct WechatClientBuilder {
    appid: Option<AppId>,
    secret: Option<AppSecret>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
}

impl WechatClientBuilder {
    /// Set the WeChat AppID
    pub fn appid(mut self, appid: AppId) -> Self {
        self.appid = Some(appid);
        self
    }

    /// Set the WeChat AppSecret
    pub fn secret(mut self, secret: AppSecret) -> Self {
        self.secret = Some(secret);
        self
    }

    /// Set the base URL for API calls
    ///
    /// Default: `<https://api.weixin.qq.com>`
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the total timeout for requests
    ///
    /// Default: 30 seconds
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the connection timeout
    ///
    /// Default: 10 seconds
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Build the WechatClient
    ///
    /// # Errors
    /// Returns an error if appid or secret is not set
    pub fn build(self) -> Result<WechatClient, WechatError> {
        let appid = self
            .appid
            .ok_or_else(|| WechatError::Config("appid is required".to_string()))?;
        let secret = self
            .secret
            .ok_or_else(|| WechatError::Config("secret is required".to_string()))?;

        let base_url = self
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let timeout = self
            .timeout
            .unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        let connect_timeout = self
            .connect_timeout
            .unwrap_or(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS));

        let client = Client::builder()
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            .build()?;

        Ok(WechatClient {
            http: client,
            appid,
            secret,
            base_url,
            middleware_executor: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default_values() {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        let client = WechatClient::builder()
            .appid(appid.clone())
            .secret(secret.clone())
            .build()
            .unwrap();

        assert_eq!(client.appid(), appid.as_str());
        assert_eq!(client.base_url(), DEFAULT_BASE_URL);
    }

    #[test]
    fn test_builder_custom_base_url() {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        let client = WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .base_url("https://custom.api.example.com")
            .build()
            .unwrap();

        assert_eq!(client.base_url(), "https://custom.api.example.com");
    }

    #[test]
    fn test_builder_custom_timeouts() {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        let client = WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        // Verify client was built successfully with custom timeouts.
        // reqwest::Client doesn't expose timeout getters, so we verify
        // the builder accepted the values and produced a valid client.
        assert_eq!(client.base_url(), DEFAULT_BASE_URL);
        assert_eq!(client.appid(), "wx1234567890abcdef");
    }

    #[test]
    fn test_builder_missing_appid() {
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        let result = WechatClient::builder().secret(secret).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_secret() {
        let appid = AppId::new("wx1234567890abcdef").unwrap();

        let result = WechatClient::builder().appid(appid).build();

        assert!(result.is_err());
    }
}
