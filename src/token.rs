//! Access token management for WeChat API
//!
//! Handles token caching, automatic refresh, and concurrency safety.
//! Implements single-flight pattern to merge concurrent token refresh requests.
//!
//! ## Features
//!
//! - Automatic token caching with configurable expiration buffer
//! - Single-flight pattern to prevent duplicate API calls
//! - Automatic retry with configurable attempts for rate-limited errors
//! - Thread-safe async implementation using tokio
//!
//! ## Usage
//!
//! ```rust,ignore
//! use wechat_mp_sdk::{WechatClient, TokenManager, types::{AppId, AppSecret}};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = WechatClient::builder()
//!         .appid(AppId::new("your_appid")?)
//!         .secret(AppSecret::new("your_secret")?)
//!         .build()?;
//!
//!     let token_manager = TokenManager::new(client);
//!
//!     // Get token (automatically fetches if not cached)
//!     let token = token_manager.get_token().await?;
//!     println!("Token: {}", token);
//!
//!     // Invalidate cache when token is revoked
//!     token_manager.invalidate().await;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! This module can return the following errors:
//! - [`WechatError::Http`] - Network request failures
//! - [`WechatError::Api`] - WeChat API errors (invalid credentials, rate limits)
//! - [`WechatError::Token`] - Token parsing or refresh failures

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::{Mutex, Notify, RwLock};

use crate::client::WechatClient;
use crate::error::{HttpError, WechatError};
use crate::types::AccessToken;
use crate::utils::jittered_delay;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;
const MAX_TOKEN_TTL_SECS: u64 = 60 * 60 * 24 * 365;

/// Retryable WeChat API error codes.
/// - -1: System busy
/// - 45009: API call limit exceeded
pub(crate) const RETRYABLE_ERROR_CODES: &[i32] = &[-1, 45009];

pub(crate) struct CachedToken {
    pub(crate) token: AccessToken,
    pub(crate) expires_at: Instant,
}

impl CachedToken {
    pub fn is_expired(&self, buffer: Duration) -> bool {
        match Instant::now().checked_add(buffer) {
            Some(deadline) => deadline >= self.expires_at,
            None => true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
    #[serde(default)]
    pub(crate) access_token: String,
    #[serde(default)]
    pub(crate) expires_in: u64,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

impl TokenResponse {
    pub(crate) fn is_success(&self) -> bool {
        self.errcode == 0
    }

    pub(crate) fn is_retryable_error(&self) -> bool {
        RETRYABLE_ERROR_CODES.contains(&self.errcode)
    }
}

type FetchResult = Result<(String, u64), WechatError>;

/// Represents an in-flight token refresh operation.
/// Multiple concurrent requests share this state and wait for the same result.
struct InFlightFetch {
    result: Arc<Mutex<Option<FetchResult>>>,
    notify: Arc<Notify>,
}

/// Manages access_token lifecycle with automatic refresh.
/// Uses single-flight pattern to merge concurrent refresh requests.
pub struct TokenManager {
    client: WechatClient,
    pub(crate) cache: Arc<RwLock<Option<CachedToken>>>,
    in_flight: Arc<Mutex<Option<Arc<InFlightFetch>>>>,
    pub(crate) refresh_buffer: Duration,
    max_retries: u32,
    retry_delay_ms: u64,
}

impl std::fmt::Debug for TokenManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenManager")
            .field("refresh_buffer", &self.refresh_buffer)
            .field("max_retries", &self.max_retries)
            .field("retry_delay_ms", &self.retry_delay_ms)
            .finish_non_exhaustive()
    }
}

impl TokenManager {
    pub fn new(client: WechatClient) -> Self {
        Self {
            client,
            cache: Arc::new(RwLock::new(None)),
            in_flight: Arc::new(Mutex::new(None)),
            refresh_buffer: Duration::from_secs(5 * 60),
            max_retries: MAX_RETRIES,
            retry_delay_ms: RETRY_DELAY_MS,
        }
    }

    pub fn builder(client: WechatClient) -> TokenManagerBuilder {
        TokenManagerBuilder::new(client)
    }

    /// Get access token, using cache if available and not expired.
    ///
    /// Automatically fetches a new token if:
    /// - No cached token exists
    /// - Cached token is expired or expiring soon (within refresh buffer)
    ///
    /// Uses single-flight pattern to merge concurrent requests.
    ///
    /// # Returns
    /// The access token string.
    ///
    /// # Errors
    /// Returns [`WechatError::Api`] if WeChat returns an error code.
    /// Returns [`WechatError::Http`] if network request fails.
    /// Returns [`WechatError::Token`] if token parsing fails.
    pub async fn get_token(&self) -> Result<String, WechatError> {
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                if !cached.is_expired(self.refresh_buffer) {
                    return Ok(cached.token.as_str().to_string());
                }
            }
        }

        let (in_flight_fetch, is_creator) = {
            let mut in_flight = self.in_flight.lock().await;

            {
                let cache = self.cache.read().await;
                if let Some(ref cached) = *cache {
                    if !cached.is_expired(self.refresh_buffer) {
                        return Ok(cached.token.as_str().to_string());
                    }
                }
            }

            match in_flight.as_ref() {
                Some(fetch) => (Arc::clone(fetch), false),
                None => {
                    let fetch = Arc::new(InFlightFetch {
                        result: Arc::new(Mutex::new(None)),
                        notify: Arc::new(Notify::new()),
                    });
                    *in_flight = Some(Arc::clone(&fetch));
                    (fetch, true)
                }
            }
        };

        if is_creator {
            self.spawn_refresh_task(Arc::clone(&in_flight_fetch));
        }

        // Wait for the shared single-flight result.
        // Creator and waiters both follow this path so cancellation of the creator
        // cannot strand the in-flight state.
        loop {
            let notified = in_flight_fetch.notify.notified();

            if let Some(ref result) = *in_flight_fetch.result.lock().await {
                return result.clone().map(|(token, _)| token);
            }

            notified.await;

            if let Some(ref result) = *in_flight_fetch.result.lock().await {
                return result.clone().map(|(token, _)| token);
            }
        }
    }

    fn spawn_refresh_task(&self, in_flight_fetch: Arc<InFlightFetch>) {
        let client = self.client.clone();
        let cache = Arc::clone(&self.cache);
        let in_flight = Arc::clone(&self.in_flight);
        let max_retries = self.max_retries;
        let retry_delay_ms = self.retry_delay_ms;

        tokio::spawn(async move {
            let fetch_result = TokenManager::fetch_token_with_retry_for_client(
                &client,
                max_retries,
                retry_delay_ms,
            )
            .await;

            let result_to_store = match fetch_result {
                Ok(TokenResponse {
                    access_token,
                    expires_in,
                    ..
                }) => AccessToken::new(access_token.as_str()).map(|_| (access_token, expires_in)),
                Err(e) => Err(e),
            };

            if let Ok((ref token_str, expires_in)) = result_to_store {
                if let Ok(token) = AccessToken::new(token_str) {
                    let ttl = Duration::from_secs(expires_in.min(MAX_TOKEN_TTL_SECS));
                    if let Some(expires_at) = Instant::now().checked_add(ttl) {
                        let cached = CachedToken { token, expires_at };
                        *cache.write().await = Some(cached);
                    }
                }
            }

            *in_flight_fetch.result.lock().await = Some(result_to_store);
            in_flight_fetch.notify.notify_waiters();
            *in_flight.lock().await = None;
        });
    }

    #[cfg(test)]
    async fn fetch_token_with_retry(&self) -> Result<TokenResponse, WechatError> {
        Self::fetch_token_with_retry_for_client(&self.client, self.max_retries, self.retry_delay_ms)
            .await
    }

    async fn fetch_token_with_retry_for_client(
        client: &WechatClient,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> Result<TokenResponse, WechatError> {
        let attempts = max_retries.max(1);
        let mut last_error = None;

        for attempt in 0..attempts {
            match Self::fetch_token_for_client(client).await {
                Ok(response) => {
                    if response.is_success() {
                        return Ok(response);
                    }
                    if response.is_retryable_error() {
                        last_error = Some(WechatError::Api {
                            code: response.errcode,
                            message: response.errmsg,
                        });
                        if attempt < attempts - 1 {
                            tokio::time::sleep(jittered_delay(retry_delay_ms, attempt)).await;
                        }
                    } else {
                        return Err(WechatError::Api {
                            code: response.errcode,
                            message: response.errmsg,
                        });
                    }
                }
                Err(WechatError::Http(e)) if e.is_transient() => {
                    last_error = Some(WechatError::Http(e));
                    if attempt < attempts - 1 {
                        tokio::time::sleep(jittered_delay(retry_delay_ms, attempt)).await;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or_else(|| WechatError::Token("Unknown error".to_string())))
    }

    async fn fetch_token_for_client(client: &WechatClient) -> Result<TokenResponse, WechatError> {
        let path = "/cgi-bin/token";
        let query = [
            ("grant_type", "client_credential"),
            ("appid", client.appid()),
            ("secret", client.secret()),
        ];
        let url = format!("{}{}", client.base_url(), path);
        let request = client.http().get(&url).query(&query).build()?;
        let response = client.send_request(request).await?;
        if let Err(error) = response.error_for_status_ref() {
            return Err(error.into());
        }

        let bytes = response.bytes().await?;
        serde_json::from_slice::<TokenResponse>(&bytes)
            .map_err(|error| WechatError::Http(HttpError::Decode(error.to_string())))
    }

    /// Invalidate cached token.
    ///
    /// Call this when you know the current access token is no longer valid
    /// (e.g., after calling the WeChat auth ticket revoke API).
    pub async fn invalidate(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
}

/// Builder for creating a `TokenManager` with custom configuration
pub struct TokenManagerBuilder {
    client: WechatClient,
    max_retries: Option<u32>,
    retry_delay_ms: Option<u64>,
    refresh_buffer_secs: Option<u64>,
}

impl TokenManagerBuilder {
    /// Create a new TokenManagerBuilder
    pub fn new(client: WechatClient) -> Self {
        Self {
            client,
            max_retries: None,
            retry_delay_ms: None,
            refresh_buffer_secs: None,
        }
    }

    /// Set the maximum number of retry attempts for token fetch
    ///
    /// Default: 3
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Set the delay in milliseconds between retry attempts
    ///
    /// Default: 100ms
    pub fn retry_delay_ms(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = Some(delay_ms);
        self
    }

    /// Set the buffer time in seconds before token expiration to trigger refresh
    ///
    /// Default: 300 seconds (5 minutes)
    pub fn refresh_buffer_secs(mut self, buffer_secs: u64) -> Self {
        self.refresh_buffer_secs = Some(buffer_secs);
        self
    }

    /// Build the TokenManager with the configured options
    pub fn build(self) -> TokenManager {
        TokenManager {
            client: self.client,
            cache: Arc::new(RwLock::new(None)),
            in_flight: Arc::new(Mutex::new(None)),
            refresh_buffer: Duration::from_secs(self.refresh_buffer_secs.unwrap_or(300)),
            max_retries: self.max_retries.unwrap_or(MAX_RETRIES),
            retry_delay_ms: self.retry_delay_ms.unwrap_or(RETRY_DELAY_MS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AppId, AppSecret};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_client() -> WechatClient {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .build()
            .unwrap()
    }

    fn create_test_client_with_base_url(base_url: &str) -> WechatClient {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .base_url(base_url)
            .build()
            .unwrap()
    }

    #[test]
    fn test_token_manager_creation() {
        let client = create_test_client();
        let manager = TokenManager::new(client);
        assert!(manager.cache.try_read().unwrap().is_none());
    }

    #[test]
    fn test_cached_token_not_expired() {
        let token = AccessToken::new("test_token").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(7200),
        };
        assert!(!cached.is_expired(Duration::from_secs(300)));
    }

    #[test]
    fn test_cached_token_expired() {
        let token = AccessToken::new("test_token").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(100),
        };
        assert!(cached.is_expired(Duration::from_secs(300)));
    }

    #[test]
    fn test_cached_token_at_boundary() {
        let token = AccessToken::new("test_token").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(300),
        };
        assert!(cached.is_expired(Duration::from_secs(300)));
    }

    #[test]
    fn test_token_response_success() {
        let response = TokenResponse {
            access_token: "token123".to_string(),
            expires_in: 7200,
            errcode: 0,
            errmsg: String::new(),
        };
        assert!(response.is_success());
    }

    #[test]
    fn test_token_response_error() {
        let response = TokenResponse {
            access_token: String::new(),
            expires_in: 0,
            errcode: 40001,
            errmsg: "invalid credential".to_string(),
        };
        assert!(!response.is_success());
    }

    #[test]
    fn test_retryable_error_code_minus_one() {
        let response = TokenResponse {
            access_token: String::new(),
            expires_in: 0,
            errcode: -1,
            errmsg: "system busy".to_string(),
        };
        assert!(response.is_retryable_error());
    }

    #[test]
    fn test_retryable_error_code_45009() {
        let response = TokenResponse {
            access_token: String::new(),
            expires_in: 0,
            errcode: 45009,
            errmsg: "api freq out of limit".to_string(),
        };
        assert!(response.is_retryable_error());
    }

    #[test]
    fn test_non_retryable_error_code() {
        let response = TokenResponse {
            access_token: String::new(),
            expires_in: 0,
            errcode: 40001,
            errmsg: "invalid credential".to_string(),
        };
        assert!(!response.is_retryable_error());
    }

    #[test]
    fn test_decode_error_is_not_transient_for_token_retry() {
        use crate::error::HttpError;

        let decode_err = HttpError::Decode("unexpected response format".to_string());
        assert!(
            !decode_err.is_transient(),
            "Decode errors must not be retried by TokenManager",
        );
    }

    #[test]
    fn test_token_response_various_errors() {
        let error_codes = [40001, 40002, 40013, 41002, 42001];
        for code in error_codes {
            let response = TokenResponse {
                access_token: String::new(),
                expires_in: 0,
                errcode: code,
                errmsg: "error".to_string(),
            };
            assert!(
                !response.is_success(),
                "Error code {} should indicate failure",
                code
            );
        }
    }

    #[tokio::test]
    async fn test_invalidate() {
        let client = create_test_client();
        let manager = TokenManager::new(client);

        let token = AccessToken::new("test").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(7200),
        };
        *manager.cache.write().await = Some(cached);

        manager.invalidate().await;

        assert!(manager.cache.read().await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_clears_cache() {
        let client = create_test_client();
        let manager = TokenManager::new(client);

        let token = AccessToken::new("test_token").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(7200),
        };
        *manager.cache.write().await = Some(cached);

        assert!(manager.cache.read().await.is_some());

        manager.invalidate().await;

        assert!(manager.cache.read().await.is_none());
    }

    #[test]
    fn test_default_refresh_buffer() {
        let client = create_test_client();
        let manager = TokenManager::new(client);
        assert_eq!(manager.refresh_buffer, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_concurrent_requests_single_api_call() {
        let mock_server = MockServer::start().await;

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = Arc::clone(&call_count);

        Mock::given(method("GET"))
            .and(path("/cgi-bin/token"))
            .and(query_param("grant_type", "client_credential"))
            .respond_with(move |_request: &wiremock::Request| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "access_token": "concurrent_test_token",
                    "expires_in": 7200,
                    "errcode": 0,
                    "errmsg": ""
                }))
            })
            .mount(&mock_server)
            .await;

        let client = create_test_client_with_base_url(&mock_server.uri());
        let manager = Arc::new(TokenManager::new(client));

        let mut handles = Vec::new();
        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            handles.push(tokio::spawn(async move { manager_clone.get_token().await }));
        }

        let results: Vec<_> = futures::future::join_all(handles).await;

        let successful_results: Vec<_> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(successful_results.len(), 10);
        for token in successful_results {
            assert_eq!(token, "concurrent_test_token");
        }

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_fetch_token_with_retry_retries_retryable_errors_then_succeeds() {
        let mock_server = MockServer::start().await;
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = Arc::clone(&call_count);

        Mock::given(method("GET"))
            .and(path("/cgi-bin/token"))
            .and(query_param("grant_type", "client_credential"))
            .respond_with(move |_request: &wiremock::Request| {
                let current = call_count_clone.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    ResponseTemplate::new(500)
                } else {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "access_token": "retry_success_token",
                        "expires_in": 7200,
                        "errcode": 0,
                        "errmsg": ""
                    }))
                }
            })
            .mount(&mock_server)
            .await;

        let client = create_test_client_with_base_url(&mock_server.uri());
        let manager = TokenManager::builder(client)
            .max_retries(3)
            .retry_delay_ms(1)
            .build();

        let response = manager.fetch_token_with_retry().await.unwrap();

        assert_eq!(response.access_token, "retry_success_token");
        assert_eq!(response.expires_in, 7200);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_fetch_token_with_retry_retries_retryable_api_error_codes_then_succeeds() {
        let mock_server = MockServer::start().await;
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = Arc::clone(&call_count);

        Mock::given(method("GET"))
            .and(path("/cgi-bin/token"))
            .and(query_param("grant_type", "client_credential"))
            .respond_with(move |_request: &wiremock::Request| {
                let current = call_count_clone.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "errcode": -1,
                        "errmsg": "system busy"
                    }))
                } else {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "access_token": "retryable_api_success_token",
                        "expires_in": 7200,
                        "errcode": 0,
                        "errmsg": "ok"
                    }))
                }
            })
            .mount(&mock_server)
            .await;

        let client = create_test_client_with_base_url(&mock_server.uri());
        let manager = TokenManager::builder(client)
            .max_retries(3)
            .retry_delay_ms(1)
            .build();

        let response = manager.fetch_token_with_retry().await.unwrap();

        assert_eq!(response.access_token, "retryable_api_success_token");
        assert_eq!(response.expires_in, 7200);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_single_flight_creator_cancellation_does_not_stall_waiters() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/cgi-bin/token"))
            .and(query_param("grant_type", "client_credential"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(120))
                    .set_body_json(serde_json::json!({
                        "access_token": "cancel_safe_token",
                        "expires_in": 7200,
                        "errcode": 0,
                        "errmsg": "ok"
                    })),
            )
            .mount(&mock_server)
            .await;

        let client = create_test_client_with_base_url(&mock_server.uri());
        let manager = Arc::new(TokenManager::new(client));

        let first = {
            let manager = Arc::clone(&manager);
            tokio::spawn(async move { manager.get_token().await })
        };

        tokio::time::sleep(Duration::from_millis(20)).await;
        first.abort();
        let _ = first.await;

        let second = tokio::time::timeout(Duration::from_millis(500), manager.get_token()).await;
        let token = second.expect("second token request timed out").unwrap();
        assert_eq!(token, "cancel_safe_token");
    }

    #[test]
    fn test_cached_token_is_expired_when_buffer_addition_overflows() {
        let token = AccessToken::new("test_token").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(1),
        };

        assert!(cached.is_expired(Duration::MAX));
    }

    #[test]
    fn test_token_manager_builder_defaults() {
        let client = create_test_client();
        let manager = TokenManager::builder(client).build();
        assert_eq!(manager.refresh_buffer, Duration::from_secs(300));
        assert_eq!(manager.max_retries, 3);
        assert_eq!(manager.retry_delay_ms, 100);
    }

    #[test]
    fn test_token_manager_builder_custom_values() {
        let client = create_test_client();
        let manager = TokenManager::builder(client)
            .max_retries(5)
            .retry_delay_ms(200)
            .refresh_buffer_secs(600)
            .build();
        assert_eq!(manager.refresh_buffer, Duration::from_secs(600));
        assert_eq!(manager.max_retries, 5);
        assert_eq!(manager.retry_delay_ms, 200);
    }

    #[test]
    fn test_token_manager_builder_partial_custom() {
        let client = create_test_client();
        let manager = TokenManager::builder(client).max_retries(10).build();
        assert_eq!(manager.max_retries, 10);
        assert_eq!(manager.retry_delay_ms, 100);
        assert_eq!(manager.refresh_buffer, Duration::from_secs(300));
    }
}
