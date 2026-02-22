//! Retry middleware for automatic retry on failures.
//!
//! This middleware automatically retries failed requests that are likely to succeed
//! on a subsequent attempt (e.g., server errors, rate limiting, or temporary failures).
//!
//! # Retry Conditions
//!
//! - HTTP 5xx responses
//! - Network errors (reqwest::Error)
//! - WeChat API error codes: -1 (system busy), 45009 (rate limit)
//!
//! # Non-Idempotent Requests
//!
//! By default, POST requests are NOT retried as they may cause duplicate operations.
//! Use `with_retry_post(true)` to enable retrying POST requests.

use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use tokio::time::sleep;
use tower::{Layer, Service};

use crate::error::WechatError;
use crate::utils::jittered_delay;

/// Middleware that retries requests on 5xx and retryable errors.
#[derive(Clone)]
pub struct RetryMiddleware {
    max_retries: usize,
    delay_ms: u64,
    retry_post: bool,
}

impl RetryMiddleware {
    /// Create a new RetryMiddleware with default settings.
    ///
    /// Default: max_retries = 3, delay_ms = 100ms, retry_post = false
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            delay_ms: 100,
            retry_post: false,
        }
    }

    /// Set maximum number of retry attempts.
    pub fn with_max_retries(mut self, max: usize) -> Self {
        self.max_retries = max;
        self
    }

    /// Set delay between retries in milliseconds.
    pub fn with_delay_ms(mut self, delay: u64) -> Self {
        self.delay_ms = delay;
        self
    }

    /// Enable retrying POST requests (disabled by default).
    pub fn with_retry_post(mut self, retry: bool) -> Self {
        self.retry_post = retry;
        self
    }

    /// Check if an error is retryable.
    ///
    /// Delegates to [`WechatError::is_transient()`] to ensure a single
    /// canonical retry-classification policy across the crate.
    pub fn is_retryable_error(error: &WechatError) -> bool {
        error.is_transient()
    }
}

impl Default for RetryMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for RetryMiddleware {
    type Service = RetryMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RetryMiddlewareService {
            inner,
            max_retries: self.max_retries,
            delay_ms: self.delay_ms,
            retry_post: self.retry_post,
        }
    }
}

#[derive(Clone)]
pub struct RetryMiddlewareService<S> {
    inner: S,
    pub(crate) max_retries: usize,
    pub(crate) delay_ms: u64,
    pub(crate) retry_post: bool,
}

/// A wrapper to identify if a request is idempotent (safe to retry).
/// This trait allows the retry middleware to work with different request types.
pub trait RetryableRequest {
    /// Returns true if the request is idempotent (GET, DELETE, etc.)
    /// POST and PUT are not idempotent by default.
    fn is_idempotent(&self) -> bool;
}

impl RetryableRequest for reqwest::Request {
    fn is_idempotent(&self) -> bool {
        !matches!(
            self.method(),
            &reqwest::Method::POST | &reqwest::Method::PUT | &reqwest::Method::PATCH
        )
    }
}

impl<S, R> Service<R> for RetryMiddlewareService<S>
where
    S: Service<R> + Send + Clone + 'static,
    S::Future: Send,
    S::Error: std::fmt::Debug + Send + From<WechatError>,
    S::Response: Send,
    R: Send + Clone + RetryableRequest + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let mut inner = self.inner.clone();
        let max_retries = self.max_retries;
        let delay_ms = self.delay_ms;
        let retry_post = self.retry_post;

        Box::pin(async move {
            // Handle max_retries=0 explicitly: no attempts should be made
            if max_retries == 0 {
                return Err(WechatError::Config(
                    "max_retries is 0: no retry attempts configured".to_string(),
                )
                .into());
            }

            let mut last_error: Option<S::Error> = None;

            // Check if request is retryable
            if !req.is_idempotent() && !retry_post {
                // Non-idempotent request and retry not enabled, try once
                return inner.call(req).await;
            }

            for attempt in 0..max_retries {
                // Clone the request for each attempt
                let req_clone = req.clone();

                match inner.call(req_clone).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        // Check if error is retryable
                        // We need to downcast the error to check if it's a WechatError
                        let is_retryable = check_error_retryable(&e);

                        if is_retryable {
                            last_error = Some(e);
                            if attempt < max_retries - 1 {
                                sleep(jittered_delay(
                                    delay_ms,
                                    u32::try_from(attempt).unwrap_or(u32::MAX),
                                ))
                                .await;
                            }
                        } else {
                            // Non-retryable error, return immediately
                            return Err(e);
                        }
                    }
                }
            }

            // At this point, max_retries > 0 and all attempts failed with retryable errors
            // last_error is guaranteed to be Some because the loop ran at least once
            Err(last_error.unwrap_or_else(|| {
                WechatError::Config("Retry exhausted without capturing error".to_string()).into()
            }))
        })
    }
}

/// Check if an error is retryable by attempting to downcast to WechatError.
fn check_error_retryable<E: std::fmt::Debug + 'static>(error: &E) -> bool {
    // Try to downcast to WechatError
    if let Some(wechat_err) = (error as &dyn std::any::Any).downcast_ref::<WechatError>() {
        return RetryMiddleware::is_retryable_error(wechat_err);
    }

    // For unknown error types, don't retry by default
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::WechatError;

    #[test]
    fn test_retry_middleware_exists() {
        let _ = RetryMiddleware::new();
    }

    #[test]
    fn test_is_retryable_error_exhaustive_variants() {
        use crate::error::HttpError;

        // Http(Reqwest) => retryable (transient transport failure)
        let reqwest_error = reqwest::Client::new().get("http://").build().unwrap_err();
        let reqwest_err = WechatError::Http(HttpError::Reqwest(std::sync::Arc::new(reqwest_error)));
        assert!(RetryMiddleware::is_retryable_error(&reqwest_err));

        // Http(Decode) => NOT retryable (response schema mismatch is not transient)
        let decode_err = WechatError::Http(HttpError::Decode("bad".into()));
        assert!(!RetryMiddleware::is_retryable_error(&decode_err));
        // Api with retryable code => retryable
        assert!(RetryMiddleware::is_retryable_error(&WechatError::Api {
            code: -1,
            message: "busy".into(),
        }));
        // Api with non-retryable code => not retryable
        assert!(!RetryMiddleware::is_retryable_error(&WechatError::Api {
            code: 40001,
            message: "invalid".into(),
        }));
        // All remaining variants => not retryable
        let non_retryable: Vec<WechatError> = vec![
            WechatError::Json(serde_json::from_str::<String>("bad").unwrap_err()),
            WechatError::Token("t".into()),
            WechatError::Config("c".into()),
            WechatError::Signature("s".into()),
            WechatError::Crypto("cr".into()),
            WechatError::InvalidAppId("a".into()),
            WechatError::InvalidOpenId("o".into()),
            WechatError::InvalidAccessToken("at".into()),
            WechatError::InvalidAppSecret("as".into()),
            WechatError::InvalidSessionKey("sk".into()),
            WechatError::InvalidUnionId("u".into()),
        ];
        for err in &non_retryable {
            assert!(
                !RetryMiddleware::is_retryable_error(err),
                "Expected non-retryable: {:?}",
                err,
            );
        }
    }

    #[test]
    fn test_retryable_error_codes() {
        let err = WechatError::Api {
            code: -1,
            message: "System busy".to_string(),
        };
        assert!(RetryMiddleware::is_retryable_error(&err));

        let err = WechatError::Api {
            code: 45009,
            message: "API limit".to_string(),
        };
        assert!(RetryMiddleware::is_retryable_error(&err));

        let err = WechatError::Api {
            code: 40001,
            message: "Invalid credential".to_string(),
        };
        assert!(!RetryMiddleware::is_retryable_error(&err));
    }

    #[test]
    fn test_decode_error_not_retryable() {
        use crate::error::HttpError;

        // Decode errors are NOT transient — a schema mismatch won't resolve on retry.
        let decode_err = WechatError::Http(HttpError::Decode("response decode error".to_string()));
        assert!(
            !RetryMiddleware::is_retryable_error(&decode_err),
            "Decode errors should not be retried",
        );
    }

    #[test]
    fn test_middleware_configuration() {
        let middleware = RetryMiddleware::new()
            .with_max_retries(5)
            .with_delay_ms(200)
            .with_retry_post(true);

        assert_eq!(middleware.max_retries, 5);
        assert_eq!(middleware.delay_ms, 200);
        assert!(middleware.retry_post);
    }

    // === Boundary Behavior Tests (TDD RED Phase) ===
    // These tests expose the last_error.unwrap() panic risk

    /// Mock request that is idempotent (safe to retry)
    #[derive(Clone)]
    struct MockIdempotentRequest;

    impl RetryableRequest for MockIdempotentRequest {
        fn is_idempotent(&self) -> bool {
            true
        }
    }

    /// Mock service that always returns a retryable error
    #[derive(Clone)]
    struct AlwaysRetryableErrorService;

    impl Service<MockIdempotentRequest> for AlwaysRetryableErrorService {
        type Response = String;
        type Error = WechatError;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: MockIdempotentRequest) -> Self::Future {
            Box::pin(async {
                Err(WechatError::Api {
                    code: -1,
                    message: "system busy".to_string(),
                })
            })
        }
    }

    /// Mock service that always returns a NON-retryable error
    #[derive(Clone)]
    struct AlwaysNonRetryableErrorService;

    impl Service<MockIdempotentRequest> for AlwaysNonRetryableErrorService {
        type Response = String;
        type Error = WechatError;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: MockIdempotentRequest) -> Self::Future {
            Box::pin(async {
                Err(WechatError::Api {
                    code: 40001,
                    message: "invalid credential".to_string(),
                })
            })
        }
    }

    /// Mock service that returns success
    #[derive(Clone)]
    struct SuccessService;

    impl Service<MockIdempotentRequest> for SuccessService {
        type Response = String;
        type Error = WechatError;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: MockIdempotentRequest) -> Self::Future {
            Box::pin(async { Ok("success".to_string()) })
        }
    }

    /// Test: max_retries=0 should NOT panic - should return error
    /// This test EXPOSES the last_error.unwrap() panic risk
    #[tokio::test]
    async fn test_max_retries_zero_should_not_panic() {
        let middleware = RetryMiddleware::new().with_max_retries(0);
        let mut service = middleware.layer(SuccessService);

        let result: Result<String, WechatError> = service.call(MockIdempotentRequest).await;

        assert!(
            result.is_err(),
            "max_retries=0 should return error, not panic"
        );
    }

    /// Test: non-retryable error should return immediately without retry
    #[tokio::test]
    async fn test_non_retryable_error_returns_immediately() {
        let middleware = RetryMiddleware::new().with_max_retries(3);
        let mut service = middleware.layer(AlwaysNonRetryableErrorService);

        let result: Result<String, WechatError> = service.call(MockIdempotentRequest).await;

        assert!(result.is_err());
        if let Err(WechatError::Api { code, .. }) = &result {
            assert_eq!(*code, 40001, "Should return the non-retryable error code");
        }
    }

    /// Test: retryable error with max_retries=1 should retry once then return error
    #[tokio::test]
    async fn test_retryable_error_with_max_retries_one() {
        let middleware = RetryMiddleware::new().with_max_retries(1).with_delay_ms(1);
        let mut service = middleware.layer(AlwaysRetryableErrorService);

        let result: Result<String, WechatError> = service.call(MockIdempotentRequest).await;

        assert!(result.is_err());
    }

    /// Test: terminal path - all retries exhausted should return last error
    #[tokio::test]
    async fn test_terminal_failure_all_retries_exhausted() {
        let middleware = RetryMiddleware::new().with_max_retries(2).with_delay_ms(1);
        let mut service = middleware.layer(AlwaysRetryableErrorService);

        let result: Result<String, WechatError> = service.call(MockIdempotentRequest).await;

        assert!(result.is_err());
        if let Err(e) = &result {
            assert!(matches!(e, WechatError::Api { code: -1, .. }));
        }
    }

    /// Test: success case - should return immediately without retry logic
    #[tokio::test]
    async fn test_success_case_no_retry() {
        let middleware = RetryMiddleware::new()
            .with_max_retries(3)
            .with_delay_ms(100);
        let mut service = middleware.layer(SuccessService);

        let result: Result<String, WechatError> = service.call(MockIdempotentRequest).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    /// Test: non-idempotent request with retry_post=false should try once only
    #[derive(Clone)]
    struct NonIdempotentRequest;

    impl RetryableRequest for NonIdempotentRequest {
        fn is_idempotent(&self) -> bool {
            false
        }
    }

    #[derive(Clone)]
    struct NonIdempotentErrorService;

    impl Service<NonIdempotentRequest> for NonIdempotentErrorService {
        type Response = String;
        type Error = WechatError;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: NonIdempotentRequest) -> Self::Future {
            Box::pin(async {
                Err(WechatError::Api {
                    code: -1,
                    message: "system busy".to_string(),
                })
            })
        }
    }

    /// Test: POST request (non-idempotent) with retry_post=false should not retry
    #[tokio::test]
    async fn test_non_idempotent_no_retry() {
        let middleware = RetryMiddleware::new()
            .with_max_retries(3)
            .with_retry_post(false);
        let mut service = middleware.layer(NonIdempotentErrorService);

        let result: Result<String, WechatError> = service.call(NonIdempotentRequest).await;

        assert!(result.is_err());
    }

    /// Test: POST request (non-idempotent) with retry_post=true should retry
    #[tokio::test]
    async fn test_non_idempotent_with_retry_enabled() {
        let middleware = RetryMiddleware::new()
            .with_max_retries(2)
            .with_delay_ms(1)
            .with_retry_post(true);
        let mut service = middleware.layer(NonIdempotentErrorService);

        let result: Result<String, WechatError> = service.call(NonIdempotentRequest).await;

        assert!(result.is_err());
    }
}
