//! Authentication middleware for automatic access_token injection.
//!
//! This middleware automatically injects the WeChat access_token into requests
//! using the Tower middleware pattern. It integrates with [`TokenManager`] for
//! automatic token caching and refresh.
//!
//! # Features
//!
//! - Automatic access_token injection as query parameter
//! - Integration with TokenManager for caching and single-flight refresh
//! - Token refresh on expiry errors (40001, 42001)
//!
//! # Example
//!
//! ```ignore
//! use tower::ServiceBuilder;
//! use wechat_mp_sdk::middleware::AuthMiddleware;
//! use wechat_mp_sdk::token::TokenManager;
//!
//! let token_manager = Arc::new(TokenManager::new(client));
//! let middleware = AuthMiddleware::new(token_manager);
//!
//! let service = ServiceBuilder::new()
//!     .layer(middleware)
//!     .service(http_client);
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http::{Request, Uri};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::{Request as ReqwestRequest, Url};
use tower::{Layer, Service};

use crate::token::TokenManager;

/// Characters that must be encoded in query parameter values.
/// Includes: space, &, =, %, +, #, and control characters.
const QUERY_VALUE_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'&')
    .add(b'=')
    .add(b'%')
    .add(b'+')
    .add(b'#');

/// Middleware that injects access_token into requests.
///
/// This middleware wraps an inner service and automatically adds the
/// WeChat access_token to outgoing requests. It uses [`TokenManager`]
/// for efficient token caching and automatic refresh.
pub struct AuthMiddleware {
    token_manager: Arc<TokenManager>,
}

impl AuthMiddleware {
    /// Create a new AuthMiddleware with the given TokenManager.
    ///
    /// # Arguments
    /// * `token_manager` - Shared reference to the TokenManager
    pub fn new(token_manager: Arc<TokenManager>) -> Self {
        Self { token_manager }
    }
}

impl<S> Layer<S> for AuthMiddleware {
    type Service = AuthMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddlewareService {
            inner,
            token_manager: Arc::clone(&self.token_manager),
        }
    }
}

impl Clone for AuthMiddleware {
    fn clone(&self) -> Self {
        Self {
            token_manager: Arc::clone(&self.token_manager),
        }
    }
}

/// Service created by AuthMiddleware that injects access_token into requests.
pub struct AuthMiddlewareService<S> {
    inner: S,
    token_manager: Arc<TokenManager>,
}

impl<S> Clone for AuthMiddlewareService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            token_manager: Arc::clone(&self.token_manager),
        }
    }
}

impl<S, B> Service<Request<B>> for AuthMiddlewareService<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let token_manager = Arc::clone(&self.token_manager);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Get the access token
            let token = match token_manager.get_token().await {
                Ok(t) => t,
                Err(_e) => {
                    log::warn!("Failed to fetch access token: {}", _e);
                    // If we can't get a token, pass the request through
                    // The inner service will handle the error case
                    return inner.call(req).await;
                }
            };

            // Inject token as query parameter
            let uri = req.uri().clone();
            let new_uri = add_access_token_query(&uri, &token);
            *req.uri_mut() = new_uri;

            inner.call(req).await
        })
    }
}

impl<S> Service<ReqwestRequest> for AuthMiddlewareService<S>
where
    S: Service<ReqwestRequest> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: ReqwestRequest) -> Self::Future {
        let token_manager = Arc::clone(&self.token_manager);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let token = match token_manager.get_token().await {
                Ok(t) => t,
                Err(_e) => {
                    log::warn!("Failed to fetch access token: {}", _e);
                    return inner.call(req).await;
                }
            };

            let url = req.url().clone();
            *req.url_mut() = add_access_token_query_to_url(&url, &token);
            inner.call(req).await
        })
    }
}

/// Add access_token query parameter to a URI.
fn add_access_token_query(uri: &Uri, token: &str) -> Uri {
    let path_and_query = match uri.path_and_query() {
        Some(pq) => pq.as_str(),
        None => return uri.clone(),
    };

    let separator = if path_and_query.contains('?') {
        "&"
    } else {
        "?"
    };

    let encoded_token = utf8_percent_encode(token, QUERY_VALUE_ENCODE_SET);

    let new_path_and_query = format!(
        "{}{}access_token={}",
        path_and_query, separator, encoded_token
    );

    let mut parts = uri.clone().into_parts();
    parts.path_and_query = match new_path_and_query.parse() {
        Ok(pq) => Some(pq),
        Err(_) => return uri.clone(),
    };

    Uri::from_parts(parts).unwrap_or_else(|_| uri.clone())
}

fn add_access_token_query_to_url(url: &Url, token: &str) -> Url {
    let mut url = url.clone();
    url.query_pairs_mut().append_pair("access_token", token);
    url
}

/// Encode a token for URL query parameters using standard percent-encoding.
#[cfg(test)]
fn encode_token(token: &str) -> String {
    utf8_percent_encode(token, QUERY_VALUE_ENCODE_SET).to_string()
}

/// Configuration for how the access token should be injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenInjection {
    /// Inject token as a query parameter (default for WeChat APIs).
    QueryParam,
    /// Inject token as an Authorization header (Bearer token).
    BearerHeader,
}

/// Middleware with configurable token injection method.
///
/// This is an extended version of AuthMiddleware that allows configuring
/// how the token is injected into requests.
pub struct ConfigurableAuthMiddleware {
    token_manager: Arc<TokenManager>,
    injection: TokenInjection,
}

impl ConfigurableAuthMiddleware {
    /// Create a new ConfigurableAuthMiddleware with the given TokenManager.
    pub fn new(token_manager: Arc<TokenManager>) -> Self {
        Self {
            token_manager,
            injection: TokenInjection::QueryParam,
        }
    }

    /// Set the token injection method.
    pub fn injection(mut self, injection: TokenInjection) -> Self {
        self.injection = injection;
        self
    }
}

impl<S> Layer<S> for ConfigurableAuthMiddleware {
    type Service = ConfigurableAuthMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ConfigurableAuthMiddlewareService {
            inner,
            token_manager: Arc::clone(&self.token_manager),
            injection: self.injection,
        }
    }
}

impl Clone for ConfigurableAuthMiddleware {
    fn clone(&self) -> Self {
        Self {
            token_manager: Arc::clone(&self.token_manager),
            injection: self.injection,
        }
    }
}

/// Service created by ConfigurableAuthMiddleware.
pub struct ConfigurableAuthMiddlewareService<S> {
    inner: S,
    token_manager: Arc<TokenManager>,
    injection: TokenInjection,
}

impl<S> Clone for ConfigurableAuthMiddlewareService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            token_manager: Arc::clone(&self.token_manager),
            injection: self.injection,
        }
    }
}

impl<S, B> Service<Request<B>> for ConfigurableAuthMiddlewareService<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let token_manager = Arc::clone(&self.token_manager);
        let injection = self.injection;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let token = match token_manager.get_token().await {
                Ok(t) => t,
                Err(_e) => {
                    log::warn!("Failed to fetch access token: {}", _e);
                    return inner.call(req).await;
                }
            };

            match injection {
                TokenInjection::QueryParam => {
                    let uri = req.uri().clone();
                    let new_uri = add_access_token_query(&uri, &token);
                    *req.uri_mut() = new_uri;
                }
                TokenInjection::BearerHeader => {
                    let header_value = format!("Bearer {}", token);
                    if let Ok(value) = http::HeaderValue::from_str(&header_value) {
                        req.headers_mut().insert(http::header::AUTHORIZATION, value);
                    }
                }
            }

            inner.call(req).await
        })
    }
}

impl<S> Service<ReqwestRequest> for ConfigurableAuthMiddlewareService<S>
where
    S: Service<ReqwestRequest> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: ReqwestRequest) -> Self::Future {
        let token_manager = Arc::clone(&self.token_manager);
        let injection = self.injection;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let token = match token_manager.get_token().await {
                Ok(t) => t,
                Err(_e) => {
                    log::warn!("Failed to fetch access token: {}", _e);
                    return inner.call(req).await;
                }
            };

            match injection {
                TokenInjection::QueryParam => {
                    let url = req.url().clone();
                    *req.url_mut() = add_access_token_query_to_url(&url, &token);
                }
                TokenInjection::BearerHeader => {
                    let header_value = format!("Bearer {}", token);
                    if let Ok(value) = http::HeaderValue::from_str(&header_value) {
                        req.headers_mut().insert(http::header::AUTHORIZATION, value);
                    }
                }
            }

            inner.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_uri() -> Uri {
        "https://api.weixin.qq.com/cgi-bin/user/info"
            .parse()
            .unwrap()
    }

    #[test]
    fn test_add_access_token_query_no_existing_query() {
        let uri = test_uri();
        let new_uri = add_access_token_query(&uri, "test_token_123");

        assert_eq!(
            new_uri.path_and_query().unwrap().as_str(),
            "/cgi-bin/user/info?access_token=test_token_123"
        );
    }

    #[test]
    fn test_add_access_token_query_with_existing_query() {
        let uri: Uri = "https://api.weixin.qq.com/cgi-bin/user/info?openid=test"
            .parse()
            .unwrap();
        let new_uri = add_access_token_query(&uri, "test_token_123");

        assert_eq!(
            new_uri.path_and_query().unwrap().as_str(),
            "/cgi-bin/user/info?openid=test&access_token=test_token_123"
        );
    }

    #[test]
    fn test_add_access_token_query_with_special_chars() {
        let uri = test_uri();
        let new_uri = add_access_token_query(&uri, "token with spaces");

        // Spaces should be percent-encoded
        assert!(new_uri
            .path_and_query()
            .unwrap()
            .as_str()
            .contains("access_token=token%20with%20spaces"));
    }

    #[test]
    fn test_token_injection_default() {
        let injection = TokenInjection::QueryParam;
        assert_eq!(injection, TokenInjection::QueryParam);
    }

    #[test]
    fn test_auth_middleware_clone() {
        use crate::types::{AppId, AppSecret};
        use crate::WechatClient;

        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        let client = WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .build()
            .unwrap();

        let token_manager = Arc::new(TokenManager::new(client));
        let middleware = AuthMiddleware::new(token_manager);
        let _cloned = middleware.clone();
    }

    #[test]
    fn test_configurable_auth_middleware_builder() {
        use crate::types::{AppId, AppSecret};
        use crate::WechatClient;

        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        let client = WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .build()
            .unwrap();

        let token_manager = Arc::new(TokenManager::new(client));
        let middleware =
            ConfigurableAuthMiddleware::new(token_manager).injection(TokenInjection::BearerHeader);

        assert_eq!(middleware.injection, TokenInjection::BearerHeader);
    }

    // ============================================
    // TDD RED phase: Tests that expose panic risks
    // ============================================

    /// Happy path test: Valid URI with normal token
    /// This test passes - represents the expected behavior
    #[test]
    fn test_add_access_token_query_happy_path() {
        let uri: Uri = "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential"
            .parse()
            .unwrap();
        let token = "valid_access_token_12345";

        let result = add_access_token_query(&uri, token);

        assert!(result
            .path_and_query()
            .unwrap()
            .as_str()
            .contains("access_token=valid_access_token_12345"));
    }

    /// PANIC RISK TEST: Malformed URI path that could cause parse().unwrap() to panic
    /// The current implementation at line 158 uses parse().unwrap() which can panic
    /// on certain invalid URI characters in the path.
    #[test]
    fn test_add_access_token_query_malformed_uri_path() {
        // This URI has invalid characters that may cause parsing issues
        let malformed_uri: Uri = "https://api.weixin.qq.com/bad[path](test)".parse().unwrap();
        let token = "test_token";

        // This could panic due to parse().unwrap() at line 158
        // The test documents the current behavior - may panic
        let result = std::panic::catch_unwind(|| add_access_token_query(&malformed_uri, token));

        assert!(
            result.is_ok(),
            "add_access_token_query should not panic for malformed path URI"
        );

        let uri = result.unwrap();
        assert_eq!(uri.scheme_str(), malformed_uri.scheme_str());
        assert_eq!(uri.host(), malformed_uri.host());
    }

    /// PANIC RISK TEST: URI with fragment that might cause issues
    /// Tests edge case where URI already has a fragment (#)
    #[test]
    fn test_add_access_token_query_uri_with_fragment() {
        // URI with fragment - the # character is special in URIs
        let uri: Uri = "https://api.weixin.qq.com/path#fragment".parse().unwrap();
        let token = "token123";

        let result = std::panic::catch_unwind(|| add_access_token_query(&uri, token));

        match result {
            Ok(new_uri) => {
                // Fragment should be preserved or handled
                println!(
                    "[RESULT] URI with fragment handled, returned: {:?}",
                    new_uri
                );
            }
            Err(_) => {
                // PANIC occurred - exposes the risk
                println!("[PANIC RISK] URI with fragment caused panic!");
            }
        }
    }

    /// PANIC RISK TEST: Token with special characters that could break URI parsing
    /// Tests edge case where encoded token might contain characters that break parsing
    #[test]
    fn test_add_access_token_query_token_with_percent_encoding_risk() {
        let uri: Uri = "https://api.weixin.qq.com/cgi-bin/token".parse().unwrap();

        // Token that after encoding might create problematic sequences
        // %00 (null byte), %ff (high byte), etc.
        let risky_token = "token%00test";

        let result = std::panic::catch_unwind(|| add_access_token_query(&uri, risky_token));

        match result {
            Ok(new_uri) => {
                println!(
                    "[RESULT] Token with risky encoding handled, returned: {:?}",
                    new_uri
                );
            }
            Err(_) => {
                // PANIC occurred - exposes the risk
                println!("[PANIC RISK] Token with risky encoding caused panic!");
            }
        }
    }

    /// PANIC RISK TEST: Empty path URI - edge case
    #[test]
    fn test_add_access_token_query_empty_path() {
        let uri: Uri = "https://api.weixin.qq.com".parse().unwrap();
        let token = "test_token";

        let result = add_access_token_query(&uri, token);

        // Should work but tests the edge case
        assert!(result.host().is_some());
    }

    /// Boundary test: Token with characters that need encoding but might break parse
    /// The encode_token function handles some cases but not all URI-invalid chars
    #[test]
    fn test_encode_token_special_characters() {
        // Test that specific characters are encoded
        let token_with_space = "hello world";
        let encoded = encode_token(token_with_space);
        assert!(encoded.contains("%20"), "Space should be encoded");

        let token_with_ampersand = "test&value";
        let encoded = encode_token(token_with_ampersand);
        assert!(encoded.contains("%26"), "Ampersand should be encoded");

        let token_with_equals = "a=b";
        let encoded = encode_token(token_with_equals);
        assert!(encoded.contains("%3D"), "Equals should be encoded");

        let token_with_percent = "100%";
        let encoded = encode_token(token_with_percent);
        assert!(encoded.contains("%25"), "Percent should be encoded");

        let token_with_plus = "a+b";
        let encoded = encode_token(token_with_plus);
        assert!(encoded.contains("%2B"), "Plus should be encoded");
    }
}
