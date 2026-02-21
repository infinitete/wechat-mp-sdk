//! HTTP Contract Tests for WechatClient
//!
//! These tests verify HTTP failure path handling for the WeChat client.
//! They cover:
//! - 4xx HTTP status codes
//! - 5xx HTTP status codes
//! - 200 OK with WeChat API error (errcode != 0)
//! - 200 OK with malformed JSON body
//!
//! This is TDD RED phase: tests should FAIL initially to expose implementation gaps.
//! Later fixes will make these GREEN.

use wechat_mp_sdk::client::WechatClient;
use wechat_mp_sdk::error::WechatError;
use wechat_mp_sdk::types::{AppId, AppSecret};
use wiremock::matchers::{method, path};
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

/// Simple response type for testing
#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
struct TokenResponse {
    access_token: String,
    expires_in: u32,
}

/// Test: 4xx HTTP status code should return Http error
///
/// Expected: WechatError::Http
/// Current (RED): May panic or return unexpected error due to no status check
#[tokio::test]
async fn test_http_4xx_status_should_return_http_error() {
    let mock_server = MockServer::start().await;

    // Mock 400 Bad Request response
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "errcode": 400,
            "errmsg": "Bad Request"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    // This should return WechatError::Http for 4xx status
    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    // Assert expected error type
    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for 4xx status, got: {:?}",
        result
    );
}

/// Test: 5xx HTTP status code should return Http error
///
/// Expected: WechatError::Http
/// Current (RED): May panic or return unexpected error due to no status check
#[tokio::test]
async fn test_http_5xx_status_should_return_http_error() {
    let mock_server = MockServer::start().await;

    // Mock 500 Internal Server Error response
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "errcode": 500,
            "errmsg": "Internal Server Error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    // This should return WechatError::Http for 5xx status
    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    // Assert expected error type
    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for 5xx status, got: {:?}",
        result
    );
}

/// Test: 401 Unauthorized HTTP status code should return Http error
///
/// Expected: WechatError::Http
/// Current (RED): May panic or return unexpected error
#[tokio::test]
async fn test_http_401_status_should_return_http_error() {
    let mock_server = MockServer::start().await;

    // Mock 401 Unauthorized response
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "errcode": 401,
            "errmsg": "Unauthorized"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for 401 status, got: {:?}",
        result
    );
}

/// Test: 403 Forbidden HTTP status code should return Http error
///
/// Expected: WechatError::Http
/// Current (RED): May panic or return unexpected error
#[tokio::test]
async fn test_http_403_status_should_return_http_error() {
    let mock_server = MockServer::start().await;

    // Mock 403 Forbidden response
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "errcode": 403,
            "errmsg": "Forbidden"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for 403 status, got: {:?}",
        result
    );
}

/// Test: 429 Too Many Requests HTTP status code should return Http error
///
/// Expected: WechatError::Http
/// Current (RED): May panic or return unexpected error
#[tokio::test]
async fn test_http_429_status_should_return_http_error() {
    let mock_server = MockServer::start().await;

    // Mock 429 Too Many Requests response
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "errcode": 429,
            "errmsg": "Rate limit exceeded"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for 429 status, got: {:?}",
        result
    );
}

/// Test: 503 Service Unavailable HTTP status code should return Http error
///
/// Expected: WechatError::Http
/// Current (RED): May panic or return unexpected error
#[tokio::test]
async fn test_http_503_status_should_return_http_error() {
    let mock_server = MockServer::start().await;

    // Mock 503 Service Unavailable response
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(503).set_body_json(serde_json::json!({
            "errcode": 503,
            "errmsg": "Service Unavailable"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for 503 status, got: {:?}",
        result
    );
}

/// Test: 200 OK with WeChat API error (errcode != 0) should return Api error
///
/// Expected: WechatError::Api { code: non-zero, message: _ }
/// Current (RED): Returns success (errcode not checked in client methods)
#[tokio::test]
async fn test_http_200_with_errcode_should_return_api_error() {
    let mock_server = MockServer::start().await;

    // Mock 200 OK but with WeChat API error (errcode = 40013 = invalid appid)
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errcode": 40013,
            "errmsg": "invalid appid"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    // This should return WechatError::Api for non-zero errcode
    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    // Assert expected error type
    assert!(
        matches!(result, Err(WechatError::Api { code: 40013, .. })),
        "Expected WechatError::Api {{ code: 40013, .. }} for non-zero errcode, got: {:?}",
        result
    );
}

/// Test: 200 OK with different WeChat API error code
///
/// Expected: WechatError::Api { code: non-zero, message: _ }
/// Current (RED): Returns success (errcode not checked)
#[tokio::test]
async fn test_http_200_with_errcode_40001_should_return_api_error() {
    let mock_server = MockServer::start().await;

    // Mock 200 OK but with errcode 40001 (invalid credential)
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errcode": 40001,
            "errmsg": "invalid credential"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Api { code: 40001, .. })),
        "Expected WechatError::Api {{ code: 40001, .. }} for non-zero errcode, got: {:?}",
        result
    );
}

/// Test: POST request with 200 OK and errcode != 0 should return Api error
///
/// Expected: WechatError::Api
/// Current (RED): Returns success (errcode not checked in post method)
#[tokio::test]
async fn test_http_post_200_with_errcode_should_return_api_error() {
    let mock_server = MockServer::start().await;

    // Mock POST request with WeChat API error
    Mock::given(method("POST"))
        .and(path("/wxa/business/getuserphonenumber"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errcode": 40001,
            "errmsg": "invalid access_token"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    #[derive(serde::Serialize)]
    struct PhoneRequest {
        code: String,
    }

    let result = client
        .post::<TokenResponse, _>(
            "/wxa/business/getuserphonenumber",
            &PhoneRequest {
                code: "test".to_string(),
            },
        )
        .await;

    assert!(
        matches!(result, Err(WechatError::Api { code: 40001, .. })),
        "Expected WechatError::Api for non-zero errcode in POST, got: {:?}",
        result
    );
}

/// Test: 200 OK with malformed JSON body should return error
///
/// Expected: WechatError::Http (reqwest wraps JSON errors as Http::Decode)
/// Current: Returns Http error with JSON decode failure
#[tokio::test]
async fn test_http_200_with_malformed_json_should_return_error() {
    let mock_server = MockServer::start().await;

    // Mock 200 OK with malformed JSON (invalid JSON syntax)
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{ invalid json }"))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected error for malformed JSON, got: {:?}",
        result
    );
}

/// Test: 200 OK with valid JSON but wrong structure should return error
///
/// Expected: WechatError::Http (reqwest wraps JSON errors)
#[tokio::test]
async fn test_http_200_with_incomplete_json_should_return_error() {
    let mock_server = MockServer::start().await;

    // Mock 200 OK with valid JSON but missing required field (access_token)
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "expires_in": 7200
            // missing access_token
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    // This should fail with error (reqwest wraps JSON errors as Http)
    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected error for missing field, got: {:?}",
        result
    );
}

/// Test: Empty response body should return error
///
/// Expected: WechatError::Http (reqwest wraps JSON errors)
#[tokio::test]
async fn test_http_200_with_empty_body_should_return_error() {
    let mock_server = MockServer::start().await;

    // Mock 200 OK with empty body
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected error for empty body, got: {:?}",
        result
    );
}

/// Test: HTML error page response should return Http or Json error
///
/// Expected: WechatError::Http or WechatError::Json
/// Current (RED): Likely returns Json error trying to parse HTML as JSON
#[tokio::test]
async fn test_http_500_with_html_body_should_return_error() {
    let mock_server = MockServer::start().await;

    // Mock 500 with HTML error page (common in production)
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_string("<html><body><h1>500 Internal Server Error</h1></body></html>")
                .append_header("content-type", "text/html"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    // Should return error (Http or Json)
    assert!(
        result.is_err(),
        "Expected error for HTML response, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_http_get_connect_failure_should_return_http_error() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let appid = AppId::new("wx1234567890abcdef".to_string()).unwrap();
    let secret = AppSecret::new("test_secret_12345".to_string()).unwrap();

    let client = WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .base_url(format!("http://127.0.0.1:{}", port))
        .timeout(std::time::Duration::from_millis(200))
        .connect_timeout(std::time::Duration::from_millis(200))
        .build()
        .unwrap();

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for connect failure, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_http_get_timeout_should_return_http_error() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        if let Ok((_socket, _)) = listener.accept().await {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let appid = AppId::new("wx1234567890abcdef".to_string()).unwrap();
    let secret = AppSecret::new("test_secret_12345".to_string()).unwrap();

    let client = WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .base_url(format!("http://{}", addr))
        .timeout(std::time::Duration::from_millis(100))
        .connect_timeout(std::time::Duration::from_millis(100))
        .build()
        .unwrap();

    let result = client.get::<TokenResponse>("/cgi-bin/token", &[]).await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for timeout, got: {:?}",
        result
    );

    let _ = server.await;
}

#[tokio::test]
async fn test_http_post_non_2xx_status_should_return_http_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/wxa/business/getuserphonenumber"))
        .respond_with(ResponseTemplate::new(502).set_body_json(serde_json::json!({
            "errcode": 502,
            "errmsg": "bad gateway"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    #[derive(serde::Serialize)]
    struct PhoneRequest {
        code: String,
    }

    let result = client
        .post::<TokenResponse, _>(
            "/wxa/business/getuserphonenumber",
            &PhoneRequest {
                code: "test".to_string(),
            },
        )
        .await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http for non-2xx POST, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_http_post_decode_error_should_return_http_decode_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/wxa/business/getuserphonenumber"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;

    #[derive(serde::Serialize)]
    struct PhoneRequest {
        code: String,
    }

    let result = client
        .post::<TokenResponse, _>(
            "/wxa/business/getuserphonenumber",
            &PhoneRequest {
                code: "test".to_string(),
            },
        )
        .await;

    assert!(
        matches!(result, Err(WechatError::Http(_))),
        "Expected WechatError::Http decode error for POST, got: {:?}",
        result
    );
}

// ============================================================
// Shared model serialization/deserialization contract tests
// ============================================================

use wechat_mp_sdk::api::common::{
    ApiResponseBase, DateRangeRequest, PaginatedRequest, PaginatedResponse, WechatApiResponse,
};

#[test]
fn test_wechat_api_response_parses_success() {
    let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
    let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
    assert_eq!(resp.errcode, 0);
    assert_eq!(resp.errmsg, "ok");
    assert!(resp.is_success());
    assert!(resp.check().is_ok());
}

#[test]
fn test_wechat_api_response_parses_error() {
    let json = r#"{"errcode": 40013, "errmsg": "invalid appid"}"#;
    let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
    assert_eq!(resp.errcode, 40013);
    assert_eq!(resp.errmsg, "invalid appid");
    assert!(!resp.is_success());
    let err = resp.check().unwrap_err();
    match err {
        WechatError::Api { code, message } => {
            assert_eq!(code, 40013);
            assert_eq!(message, "invalid appid");
        }
        other => panic!("Expected WechatError::Api, got: {:?}", other),
    }
}

#[test]
fn test_paginated_response_parses_correctly() {
    #[derive(serde::Deserialize, Debug)]
    struct TestItem {
        id: u32,
        name: String,
    }

    let json = r#"{
        "total_count": 50,
        "list": [
            {"id": 1, "name": "alpha"},
            {"id": 2, "name": "beta"},
            {"id": 3, "name": "gamma"}
        ],
        "errcode": 0,
        "errmsg": "ok"
    }"#;

    let resp: PaginatedResponse<TestItem> = serde_json::from_str(json).unwrap();
    assert_eq!(resp.total_count, 50);
    assert_eq!(resp.list.len(), 3);
    assert_eq!(resp.list[0].id, 1);
    assert_eq!(resp.list[0].name, "alpha");
    assert_eq!(resp.list[2].id, 3);
    assert_eq!(resp.list[2].name, "gamma");
    assert!(resp.is_success());
    assert!(resp.check().is_ok());
}

#[test]
fn test_malformed_payload_returns_decode_error() {
    let malformed_json = "{ not valid json }";
    let result = serde_json::from_str::<ApiResponseBase>(malformed_json);
    assert!(result.is_err(), "Expected decode error for malformed JSON");
}

#[test]
fn test_paginated_response_error_with_no_list() {
    #[derive(serde::Deserialize, Debug)]
    struct TestItem {
        #[allow(dead_code)]
        name: String,
    }

    let json = r#"{"errcode": 40001, "errmsg": "invalid credential"}"#;
    let resp: PaginatedResponse<TestItem> = serde_json::from_str(json).unwrap();
    assert!(!resp.is_success());
    assert!(resp.list.is_empty());
    assert_eq!(resp.total_count, 0);
    assert!(resp.check().is_err());
}

#[test]
fn test_paginated_request_serializes_correctly() {
    let req = PaginatedRequest::new(10, 20);
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["offset"], 10);
    assert_eq!(json["count"], 20);
}

#[test]
fn test_date_range_request_serializes_correctly() {
    let req = DateRangeRequest::new("20240101", "20240131");
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["begin_date"], "20240101");
    assert_eq!(json["end_date"], "20240131");
}

#[test]
fn test_api_response_base_extra_fields_ignored() {
    let json = r#"{"errcode": 0, "errmsg": "ok", "extra": "ignored"}"#;
    let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
    assert!(resp.is_success());
}

#[test]
fn test_api_response_base_missing_fields_default() {
    let json = r#"{}"#;
    let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
    assert_eq!(resp.errcode, 0);
    assert!(resp.errmsg.is_empty());
    assert!(resp.is_success());
}
