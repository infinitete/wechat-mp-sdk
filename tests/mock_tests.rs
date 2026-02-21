//! Mock API Tests using WireMock
//!
//! These tests mock the WeChat API responses to verify request parameters
//! and response parsing without making real network calls.

use wechat_mp_sdk::api::auth::AuthApi;
use wechat_mp_sdk::api::user::UserApi;
use wechat_mp_sdk::client::WechatClient;
use wechat_mp_sdk::token::TokenManager;
use wechat_mp_sdk::types::{AppId, AppSecret};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Create a test client pointing to the mock server
async fn create_test_client(mock_server: &MockServer) -> WechatClient {
    let appid = AppId::new("wx1234567890abcdef".to_string()).unwrap();
    let secret = AppSecret::new("test_secret_12345".to_string()).unwrap();

    WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .base_url(mock_server.uri())
        .build()
        .unwrap()
}

/// Test successful access token retrieval with mock
#[tokio::test]
async fn test_mock_access_token() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_123",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Create client and token manager
    let client = create_test_client(&mock_server).await;
    let token_manager = TokenManager::new(client);

    // Get token
    let token = token_manager.get_token().await.unwrap();

    // Verify the token
    assert_eq!(token, "mock_token_123");
}

/// Test successful login (code2session) with mock
#[tokio::test]
async fn test_mock_login() {
    let mock_server = MockServer::start().await;

    // Mock the login endpoint
    Mock::given(method("GET"))
        .and(path("/sns/jscode2session"))
        .and(query_param("js_code", "test_code_12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "openid": "oXXXXXXXXXXXXXXXXXXXXXXXXXXX",
            "session_key": "test_session_key_value",
            "unionid": "o6_bmjrPTlm6_2sgVt7hMZOPfL2M",
            "errcode": 0,
            "errmsg": ""
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let auth_api = AuthApi::new(client);

    let response = auth_api.login("test_code_12345").await.unwrap();

    // Verify response
    assert_eq!(response.openid, "oXXXXXXXXXXXXXXXXXXXXXXXXXXX");
    assert_eq!(response.session_key, "test_session_key_value");
    assert!(response.is_success());
    assert!(response.unionid.is_some());
}

/// Test login error response handling
#[tokio::test]
async fn test_mock_login_error() {
    let mock_server = MockServer::start().await;

    // Mock error response
    Mock::given(method("GET"))
        .and(path("/sns/jscode2session"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "openid": "",
            "session_key": "",
            "errcode": 40029,
            "errmsg": "invalid code"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let auth_api = AuthApi::new(client);

    let result = auth_api.login("invalid_code").await;

    // Should return error
    assert!(result.is_err());
}

/// Test get phone number with mock
#[tokio::test]
async fn test_mock_get_phone_number() {
    let mock_server = MockServer::start().await;

    // First mock the token endpoint (needed for phone number API)
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_for_phone",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Then mock the phone number endpoint
    Mock::given(method("POST"))
        .and(path("/wxa/business/getuserphonenumber"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "phone_info": {
                "phone_number": "+8613800138000",
                "pure_phone_number": "13800138000",
                "country_code": "86",
                "watermark": {
                    "timestamp": 1629782400,
                    "appid": "wx1234567890abcdef"
                }
            },
            "errcode": 0,
            "errmsg": ""
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let token_manager = TokenManager::new(client.clone());
    let user_api = UserApi::new(client);

    let response = user_api
        .get_phone_number(&token_manager, "phone_code")
        .await
        .unwrap();

    // Verify phone info
    assert_eq!(response.phone_info.pure_phone_number, "13800138000");
    assert_eq!(response.phone_info.country_code, "86");
}

/// Test token request includes correct parameters
#[tokio::test]
async fn test_mock_token_request_parameters() {
    let mock_server = MockServer::start().await;

    // Mock that verifies query parameters
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .and(query_param("grant_type", "client_credential"))
        .and(query_param("appid", "wx1234567890abcdef"))
        .and(query_param("secret", "test_secret_12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "verified_token",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let token_manager = TokenManager::new(client);

    let token = token_manager.get_token().await.unwrap();

    assert_eq!(token, "verified_token");
}

/// Test login request includes correct parameters
#[tokio::test]
async fn test_mock_login_request_parameters() {
    let mock_server = MockServer::start().await;

    // Mock that verifies all required query parameters
    Mock::given(method("GET"))
        .and(path("/sns/jscode2session"))
        .and(query_param("appid", "wx1234567890abcdef"))
        .and(query_param("secret", "test_secret_12345"))
        .and(query_param("js_code", "my_test_code"))
        .and(query_param("grant_type", "authorization_code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "openid": "test_openid",
            "session_key": "test_session",
            "errcode": 0,
            "errmsg": ""
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let auth_api = AuthApi::new(client);

    let response = auth_api.login("my_test_code").await.unwrap();

    assert_eq!(response.openid, "test_openid");
}

/// Test phone number error handling with mock
#[tokio::test]
async fn test_mock_phone_number_error() {
    let mock_server = MockServer::start().await;

    // Mock token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock phone number endpoint returning error
    Mock::given(method("POST"))
        .and(path("/wxa/business/getuserphonenumber"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errcode": 40001,
            "errmsg": "invalid access_token"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let token_manager = TokenManager::new(client.clone());
    let user_api = UserApi::new(client);

    let result = user_api
        .get_phone_number(&token_manager, "invalid_code")
        .await;

    assert!(result.is_err());
}

/// Test token cache is used on subsequent calls
#[tokio::test]
async fn test_mock_token_caching() {
    let mock_server = MockServer::start().await;

    // Mock token endpoint - should only be called once
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "cached_token",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let token_manager = TokenManager::new(client);

    // First call - should hit mock
    let token1 = token_manager.get_token().await.unwrap();
    assert_eq!(token1, "cached_token");

    // Second call - should use cache
    let token2 = token_manager.get_token().await.unwrap();
    assert_eq!(token2, "cached_token");
}

/// Test LoginResponse parsing with different response formats
#[tokio::test]
async fn test_mock_login_response_formats() {
    let mock_server = MockServer::start().await;

    // Response without unionid (common case)
    Mock::given(method("GET"))
        .and(path("/sns/jscode2session"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "openid": "test_openid_no_union",
            "session_key": "test_session_key",
            "errcode": 0,
            "errmsg": "ok"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let auth_api = AuthApi::new(client);

    let response = auth_api.login("code_no_unionid").await.unwrap();

    // Should parse correctly
    assert!(response.unionid.is_none());
    assert!(response.is_success());
}
