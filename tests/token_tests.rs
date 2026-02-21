use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use wechat_mp_sdk::token::{CachedToken, TokenManager, TokenResponse};
use wechat_mp_sdk::types::{AccessToken, AppId, AppSecret};
use wechat_mp_sdk::WechatClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn create_test_client() -> WechatClient {
    let appid = AppId::new("wx1234567890abcdef").unwrap();
    let secret = AppSecret::new("secret1234567890ab".to_string()).unwrap();
    WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .build()
        .unwrap()
}

#[test]
fn test_token_manager_creation() {
    let client = create_test_client();
    let manager = TokenManager::new(client);
    let cache = manager.cache.try_read().unwrap();
    assert!(cache.is_none());
}

#[test]
fn test_cached_token_not_expired() {
    let token = AccessToken::new("test_token_12345".to_string()).unwrap();
    let cached = CachedToken {
        token,
        expires_at: Instant::now() + Duration::from_secs(7200),
    };
    assert!(!cached.is_expired(Duration::from_secs(300)));
}

#[test]
fn test_cached_token_expired() {
    let token = AccessToken::new("test_token_12345".to_string()).unwrap();
    let cached = CachedToken {
        token,
        expires_at: Instant::now() + Duration::from_secs(100),
    };
    assert!(cached.is_expired(Duration::from_secs(300)));
}

#[test]
fn test_cached_token_at_boundary() {
    let token = AccessToken::new("test_token".to_string()).unwrap();
    let cached = CachedToken {
        token,
        expires_at: Instant::now() + Duration::from_secs(300),
    };
    assert!(cached.is_expired(Duration::from_secs(300)));
}

#[test]
fn test_token_response_success() {
    let response = TokenResponse {
        access_token: "test_access_token_abc".to_string(),
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
        errmsg: "invalid credential, access_token is invalid or not latest".to_string(),
    };
    assert!(!response.is_success());
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
async fn test_invalidate_clears_cache() {
    let client = create_test_client();
    let manager = TokenManager::new(client);

    let token = AccessToken::new("test_token".to_string()).unwrap();
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

#[test]
fn test_access_token_new() {
    let token = AccessToken::new("test_token_value".to_string()).unwrap();
    assert_eq!(token.as_str(), "test_token_value");
}

#[test]
fn test_access_token_various_formats() {
    let tokens = [
        "abc123",
        "ABCDEF1234567890abcdef",
        "abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    ];

    for token_str in tokens {
        let token = AccessToken::new(token_str.to_string());
        assert!(token.is_ok(), "Token '{}' should be valid", token_str);
    }
}

#[tokio::test]
async fn test_concurrent() {
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

    let appid = AppId::new("wx1234567890abcdef").unwrap();
    let secret = AppSecret::new("secret1234567890ab").unwrap();
    let base_url = mock_server.uri();
    let client = WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .base_url(&base_url)
        .build()
        .unwrap();

    let manager = Arc::new(TokenManager::new(client));

    let manager1 = Arc::clone(&manager);
    let manager2 = Arc::clone(&manager);
    let manager3 = Arc::clone(&manager);
    let manager4 = Arc::clone(&manager);
    let manager5 = Arc::clone(&manager);

    let (r1, r2, r3, r4, r5) = tokio::join!(
        manager1.get_token(),
        manager2.get_token(),
        manager3.get_token(),
        manager4.get_token(),
        manager5.get_token()
    );

    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());
    assert!(r4.is_ok());
    assert!(r5.is_ok());

    assert_eq!(r1.unwrap(), "concurrent_test_token");
    assert_eq!(r2.unwrap(), "concurrent_test_token");
    assert_eq!(r3.unwrap(), "concurrent_test_token");
    assert_eq!(r4.unwrap(), "concurrent_test_token");
    assert_eq!(r5.unwrap(), "concurrent_test_token");

    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}
