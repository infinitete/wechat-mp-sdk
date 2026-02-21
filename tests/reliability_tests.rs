//! Reliability tests for token management: retry, rate-limit, and token-race behavior.
//!
//! Verifies the existing token management and retry infrastructure works correctly
//! under concurrent load. Covers single-flight, caching, invalidation, HTTP retry,
//! error-code handling, retry budget exhaustion, and race safety.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use wechat_mp_sdk::client::WechatClient;
use wechat_mp_sdk::token::TokenManager;
use wechat_mp_sdk::types::{AppId, AppSecret};
use wechat_mp_sdk::WechatError;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_test_client(base_url: &str) -> WechatClient {
    let appid = AppId::new("wx1234567890abcdef").unwrap();
    let secret = AppSecret::new("secret1234567890ab").unwrap();
    WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .base_url(base_url)
        .build()
        .unwrap()
}

fn token_success_json(token: &str) -> serde_json::Value {
    serde_json::json!({
        "access_token": token,
        "expires_in": 7200
    })
}

fn token_error_json(errcode: i32, errmsg: &str) -> serde_json::Value {
    serde_json::json!({
        "errcode": errcode,
        "errmsg": errmsg
    })
}

fn token_mock() -> wiremock::matchers::MethodExactMatcher {
    method("GET")
}

fn token_path() -> wiremock::matchers::PathExactMatcher {
    path("/cgi-bin/token")
}

// ============================================================
// 1. Single-Flight Tests
// ============================================================

/// 20 concurrent `get_token()` calls via `tokio::spawn` produce exactly 1 HTTP request.
/// Uses wiremock `.expect(1)` for deterministic validation — the mock server panics on
/// drop if the hit count is wrong.
#[tokio::test]
async fn test_single_flight_concurrent_spawns_one_http_call() {
    let mock_server = MockServer::start().await;

    Mock::given(token_mock())
        .and(token_path())
        .and(query_param("grant_type", "client_credential"))
        .respond_with(ResponseTemplate::new(200).set_body_json(token_success_json("sf_token_abc")))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = Arc::new(TokenManager::new(client));

    let mut handles = Vec::new();
    for _ in 0..20 {
        let m = Arc::clone(&manager);
        handles.push(tokio::spawn(async move { m.get_token().await }));
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    for (i, result) in results.iter().enumerate() {
        let token = result
            .as_ref()
            .unwrap_or_else(|e| panic!("Task {i} panicked: {e:?}"))
            .as_ref()
            .unwrap_or_else(|e| panic!("Task {i} returned error: {e:?}"));
        assert_eq!(token, "sf_token_abc", "Task {i} got wrong token");
    }
}

// ============================================================
// 2. Token Caching Tests
// ============================================================

/// Sequential `get_token()` calls within TTL reuse cached token — no additional HTTP request.
#[tokio::test]
async fn test_token_caching_no_refetch_within_ttl() {
    let mock_server = MockServer::start().await;

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(
            ResponseTemplate::new(200).set_body_json(token_success_json("cached_token_xyz")),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::new(client);

    let first = manager.get_token().await.unwrap();
    let second = manager.get_token().await.unwrap();
    let third = manager.get_token().await.unwrap();

    assert_eq!(first, "cached_token_xyz");
    assert_eq!(second, "cached_token_xyz");
    assert_eq!(third, "cached_token_xyz");
}

// ============================================================
// 3. Token Invalidation Tests
// ============================================================

/// `invalidate()` clears the cache, forcing the next `get_token()` to re-fetch.
#[tokio::test]
async fn test_invalidate_forces_refetch() {
    let mock_server = MockServer::start().await;
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = Arc::clone(&call_count);

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(move |_req: &wiremock::Request| {
            let n = cc.fetch_add(1, Ordering::SeqCst);
            let token = format!("token_v{}", n + 1);
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": token,
                "expires_in": 7200
            }))
        })
        .expect(2)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::new(client);

    let first = manager.get_token().await.unwrap();
    assert_eq!(first, "token_v1");

    manager.invalidate().await;

    let second = manager.get_token().await.unwrap();
    assert_eq!(second, "token_v2");

    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

/// After invalidation, a cached call still returns the refreshed value.
#[tokio::test]
async fn test_invalidate_then_cache_hit() {
    let mock_server = MockServer::start().await;
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = Arc::clone(&call_count);

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(move |_req: &wiremock::Request| {
            let n = cc.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": format!("tok_{}", n + 1),
                "expires_in": 7200
            }))
        })
        .expect(2)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::new(client);

    let _ = manager.get_token().await.unwrap();

    manager.invalidate().await;
    let refreshed = manager.get_token().await.unwrap();
    assert_eq!(refreshed, "tok_2");

    let cached = manager.get_token().await.unwrap();
    assert_eq!(cached, "tok_2");

    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

// ============================================================
// 4. Retry on HTTP Errors Tests
// ============================================================

/// HTTP 5xx errors are retried. First 2 requests return 500, third returns success.
#[tokio::test]
async fn test_retry_on_http_5xx_then_success() {
    let mock_server = MockServer::start().await;
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = Arc::clone(&call_count);

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(move |_req: &wiremock::Request| {
            let n = cc.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                ResponseTemplate::new(500)
            } else {
                ResponseTemplate::new(200).set_body_json(token_success_json("retry_success_token"))
            }
        })
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::builder(client)
        .max_retries(3)
        .retry_delay_ms(1)
        .build();

    let token = manager.get_token().await.unwrap();
    assert_eq!(token, "retry_success_token");
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
}

/// A single transient HTTP 503 followed by success — verifies minimal-retry path.
#[tokio::test]
async fn test_retry_single_503_then_success() {
    let mock_server = MockServer::start().await;
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = Arc::clone(&call_count);

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(move |_req: &wiremock::Request| {
            let n = cc.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                ResponseTemplate::new(503)
            } else {
                ResponseTemplate::new(200).set_body_json(token_success_json("recovered_token"))
            }
        })
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::builder(client)
        .max_retries(3)
        .retry_delay_ms(1)
        .build();

    let token = manager.get_token().await.unwrap();
    assert_eq!(token, "recovered_token");
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

// ============================================================
// 5. Error Code Handling Tests
// ============================================================

/// Errcode -1 (system busy) is surfaced as `WechatError::Api`.
#[tokio::test]
async fn test_errcode_minus_one_returns_api_error() {
    let mock_server = MockServer::start().await;

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(
            ResponseTemplate::new(200).set_body_json(token_error_json(-1, "system error")),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::builder(client)
        .max_retries(3)
        .retry_delay_ms(1)
        .build();

    let result = manager.get_token().await;
    assert!(result.is_err());

    match result.unwrap_err() {
        WechatError::Api { code, .. } => assert_eq!(code, -1),
        other => panic!("Expected WechatError::Api with code -1, got: {other:?}"),
    }
}

/// Errcode 45009 (rate limit exceeded) is surfaced as `WechatError::Api`.
#[tokio::test]
async fn test_errcode_45009_rate_limit_returns_api_error() {
    let mock_server = MockServer::start().await;

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(token_error_json(45009, "reach max api daily quota limit")),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::builder(client)
        .max_retries(3)
        .retry_delay_ms(1)
        .build();

    let result = manager.get_token().await;
    assert!(result.is_err());

    match result.unwrap_err() {
        WechatError::Api { code, message } => {
            assert_eq!(code, 45009);
            assert!(message.contains("quota"), "Unexpected message: {message}");
        }
        other => panic!("Expected WechatError::Api with code 45009, got: {other:?}"),
    }
}

/// Non-retryable errcode (40001 invalid credential) returns error without retry.
#[tokio::test]
async fn test_non_retryable_errcode_returns_immediately() {
    let mock_server = MockServer::start().await;

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(
            ResponseTemplate::new(200).set_body_json(token_error_json(40001, "invalid credential")),
        )
        .expect(1) // Only 1 HTTP call — no retries
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::builder(client)
        .max_retries(3)
        .retry_delay_ms(1)
        .build();

    let result = manager.get_token().await;
    assert!(result.is_err());

    match result.unwrap_err() {
        WechatError::Api { code, .. } => assert_eq!(code, 40001),
        other => panic!("Expected WechatError::Api with code 40001, got: {other:?}"),
    }
}

// ============================================================
// 6. Retry Budget Exhaustion Tests
// ============================================================

/// After `max_retries` HTTP failures, `get_token()` returns a stable error.
/// All retry attempts are consumed.
#[tokio::test]
async fn test_retry_budget_exhaustion_returns_error() {
    let mock_server = MockServer::start().await;
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = Arc::clone(&call_count);

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(move |_req: &wiremock::Request| {
            cc.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(500)
        })
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::builder(client)
        .max_retries(3)
        .retry_delay_ms(1)
        .build();

    let result = manager.get_token().await;
    assert!(result.is_err());
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        3,
        "All 3 retries consumed"
    );
}

/// With max_retries=1, exactly 1 attempt is made before returning error.
#[tokio::test]
async fn test_retry_budget_single_attempt() {
    let mock_server = MockServer::start().await;

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::builder(client)
        .max_retries(1)
        .retry_delay_ms(1)
        .build();

    let result = manager.get_token().await;
    assert!(result.is_err());
}

// ============================================================
// 7. Token Invalidation Race Safety Tests
// ============================================================

/// Concurrent `get_token()` + `invalidate()` operations complete without panic or deadlock.
#[tokio::test]
async fn test_invalidation_race_safety_no_panic() {
    let mock_server = MockServer::start().await;
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = Arc::clone(&call_count);

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(move |_req: &wiremock::Request| {
            let n = cc.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": format!("race_token_{}", n),
                "expires_in": 7200
            }))
        })
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = Arc::new(TokenManager::new(client));

    let mut handles = Vec::new();

    for i in 0..20 {
        let m = Arc::clone(&manager);
        if i % 5 == 0 {
            handles.push(tokio::spawn(async move {
                m.invalidate().await;
                Ok::<String, WechatError>("invalidated".to_string())
            }));
        } else {
            handles.push(tokio::spawn(async move { m.get_token().await }));
        }
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Task {i} panicked: {:?}",
            result.as_ref().err()
        );

        assert!(
            result.as_ref().unwrap().is_ok(),
            "Task {i} returned error: {:?}",
            result.as_ref().unwrap().as_ref().err()
        );
    }

    assert!(
        call_count.load(Ordering::SeqCst) >= 1,
        "Expected at least 1 HTTP call"
    );
}

/// Rapid invalidation between get_token calls never causes stale reads.
#[tokio::test]
async fn test_invalidation_consistency() {
    let mock_server = MockServer::start().await;
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = Arc::clone(&call_count);

    Mock::given(token_mock())
        .and(token_path())
        .respond_with(move |_req: &wiremock::Request| {
            let n = cc.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": format!("version_{}", n + 1),
                "expires_in": 7200
            }))
        })
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let manager = TokenManager::new(client);

    let t1 = manager.get_token().await.unwrap();
    assert_eq!(t1, "version_1");

    manager.invalidate().await;
    let t2 = manager.get_token().await.unwrap();
    assert_eq!(t2, "version_2");

    manager.invalidate().await;
    let t3 = manager.get_token().await.unwrap();
    assert_eq!(t3, "version_3");

    assert_eq!(call_count.load(Ordering::SeqCst), 3);
}
