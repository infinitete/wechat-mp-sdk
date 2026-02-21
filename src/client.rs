//! WeChat HTTP Client
//!
//! Provides HTTP client wrapper for WeChat API calls.

use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::error::WechatError;
use crate::types::{AppId, AppSecret};

const DEFAULT_BASE_URL: &str = "https://api.weixin.qq.com";
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

/// WeChat API Client
///
/// Reusable HTTP client for calling WeChat APIs.
/// Built with reqwest for async HTTP requests.
#[allow(dead_code)]
pub struct WechatClient {
    http: Client,
    appid: AppId,
    secret: AppSecret,
    base_url: String,
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
    pub fn secret(&self) -> &str {
        self.secret.as_str()
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the underlying HTTP client for raw requests
    pub fn http(&self) -> &Client {
        &self.http
    }

    /// Make a GET request to WeChat API
    ///
    /// # Arguments
    /// * `path` - API endpoint path (e.g., "/cgi-bin/token")
    /// * `query` - Query parameters as key-value pairs
    ///
    /// # Returns
    /// Deserialized response of type T
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, WechatError> {
        let url = format!("{}{}", self.base_url, path);

        let request = self.http.get(url).query(query);

        let response = request.send().await?;

        let result = response.json::<T>().await?;

        Ok(result)
    }

    /// Make a POST request to WeChat API
    ///
    /// # Arguments
    /// * `path` - API endpoint path (e.g., "/wxa/getwxadevinfo")
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Returns
    /// Deserialized response of type T
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, WechatError> {
        let url = format!("{}{}", self.base_url, path);

        let request = self.http.post(url).json(body);

        let response = request.send().await?;

        let result = response.json::<T>().await?;

        Ok(result)
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
#[derive(Default)]
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
    /// Default: "https://api.weixin.qq.com"
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

        assert_eq!(client.base_url(), DEFAULT_BASE_URL);
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
