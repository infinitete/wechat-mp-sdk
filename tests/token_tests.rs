use std::time::{Duration, Instant};

use wechat_mp_sdk::token::{CachedToken, TokenManager, TokenResponse};
use wechat_mp_sdk::types::{AccessToken, AppId, AppSecret};
use wechat_mp_sdk::WechatClient;

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
    let cache = manager.cache.try_lock().unwrap();
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
    *manager.cache.lock().await = Some(cached);

    assert!(manager.cache.lock().await.is_some());

    manager.invalidate().await;

    assert!(manager.cache.lock().await.is_none());
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
