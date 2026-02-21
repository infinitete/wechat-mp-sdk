//! QRCode API Tests using WireMock
//!
//! These tests mock the WeChat API responses to verify QRCode API functionality
//! without making real network calls.

use std::sync::Arc;

use wechat_mp_sdk::api::qrcode::{
    LineColor, QrcodeApi, QrcodeOptions, ShortLinkOptions, UnlimitQrcodeOptions, UrlLinkOptions,
    UrlSchemeOptions,
};
use wechat_mp_sdk::api::WechatContext;
use wechat_mp_sdk::client::WechatClient;
use wechat_mp_sdk::token::TokenManager;
use wechat_mp_sdk::types::{AppId, AppSecret};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn create_test_context(mock_server: &MockServer) -> Arc<WechatContext> {
    let appid = AppId::new("wx1234567890abcdef").unwrap();
    let secret = AppSecret::new("test_secret_12345").unwrap();
    let client = WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .base_url(mock_server.uri())
        .build()
        .unwrap();
    let token_manager = TokenManager::new(client.clone());
    Arc::new(WechatContext::new(
        Arc::new(client),
        Arc::new(token_manager),
    ))
}

/// Test successful get_wxa_code_unlimit with mock
#[tokio::test]
async fn test_mock_get_wxa_code_unlimit_success() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock the getwxacodeunlimit endpoint - return a small valid image (1x1 PNG)
    // This is a minimal valid PNG file (1x1 transparent pixel)
    let valid_png: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    Mock::given(method("POST"))
        .and(path("/wxa/getwxacodeunlimit"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(valid_png))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let mut options = UnlimitQrcodeOptions::new("abc123");
    options.page = Some("/pages/index/index".to_string());
    options.width = Some(430);
    options.auto_color = Some(false);
    options.line_color = Some(LineColor { r: 0, g: 0, b: 0 });
    options.is_hyaline = Some(false);

    let result = qrcode_api.get_wxa_code_unlimit(options).await;

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert!(bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
    assert!(!bytes.is_empty());
}

/// Test get_wxa_code_unlimit error response handling
#[tokio::test]
async fn test_mock_get_wxa_code_unlimit_error() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock error response from WeChat API
    Mock::given(method("POST"))
        .and(path("/wxa/getwxacodeunlimit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errcode": 45009,
            "errmsg": "Frequency limit reached"
        })))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let mut options = UnlimitQrcodeOptions::new("test_scene");
    options.width = Some(430);

    let result = qrcode_api.get_wxa_code_unlimit(options).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, wechat_mp_sdk::WechatError::Api { .. }));
}

/// Test successful get_wxa_code with mock
#[tokio::test]
async fn test_mock_get_wxa_code_success() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock the getwxacode endpoint - return valid PNG
    let valid_png: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    Mock::given(method("POST"))
        .and(path("/wxa/getwxacode"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(valid_png))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let mut options = QrcodeOptions::new();
    options.path = Some("/pages/index/index".to_string());
    options.width = Some(430);
    options.auto_color = Some(false);
    options.line_color = Some(LineColor { r: 0, g: 0, b: 0 });
    options.is_hyaline = Some(false);

    let result = qrcode_api.get_wxa_code(options).await;

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert!(bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
}

#[tokio::test]
async fn test_mock_create_qrcode_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    let expected_body = serde_json::json!({
        "path": "/pages/index/index",
        "width": 430
    });

    Mock::given(method("POST"))
        .and(path("/cgi-bin/wxaapp/createwxaqrcode"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
        )
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let result = qrcode_api
        .create_qrcode("/pages/index/index", Some(430))
        .await;

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert!(bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
}

#[tokio::test]
async fn test_mock_create_qrcode_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/cgi-bin/wxaapp/createwxaqrcode"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_json(serde_json::json!({
                    "errcode": 45009,
                    "errmsg": "reach max api daily quota limit"
                })),
        )
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let result = qrcode_api.create_qrcode("/pages/index", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, wechat_mp_sdk::WechatError::Api { .. }));
}

/// Test successful generate_url_scheme with mock
#[tokio::test]
async fn test_mock_generate_url_scheme_success() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock the generatescheme endpoint
    Mock::given(method("POST"))
        .and(path("/wxa/generatescheme"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "openlink": "weixin://dl/business/?t=XXXXXX",
            "errcode": 0,
            "errmsg": "ok"
        })))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let options = UrlSchemeOptions {
        path: Some("/pages/index/index".to_string()),
        query: Some("id=123".to_string()),
        expire: None,
    };

    let result = qrcode_api.generate_url_scheme(options).await;

    assert!(result.is_ok());
    let scheme_url = result.unwrap();
    assert!(scheme_url.starts_with("weixin://dl/business/"));
}

/// Test generate_url_scheme error response handling
#[tokio::test]
async fn test_mock_generate_url_scheme_error() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock error response
    Mock::given(method("POST"))
        .and(path("/wxa/generatescheme"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errcode": 40097,
            "errmsg": "invalid page path"
        })))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let options = UrlSchemeOptions {
        path: Some("/invalid/path".to_string()),
        query: None,
        expire: None,
    };

    let result = qrcode_api.generate_url_scheme(options).await;

    assert!(result.is_err());
}

/// Test successful generate_url_link with mock
#[tokio::test]
async fn test_mock_generate_url_link_success() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock the generate_urllink endpoint
    Mock::given(method("POST"))
        .and(path("/wxa/generate_urllink"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "link": "https://wxaurl.cn/XXXXXX",
            "errcode": 0,
            "errmsg": "ok"
        })))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let options = UrlLinkOptions {
        path: Some("/pages/index/index".to_string()),
        query: Some("id=456".to_string()),
        expire_type: Some(1),
        expire_time: Some(1672531200),
        expire_interval: None,
    };

    let result = qrcode_api.generate_url_link(options).await;

    assert!(result.is_ok());
    let link = result.unwrap();
    assert!(link.starts_with("https://wxaurl.cn/"));
}

/// Test generate_url_link error response handling
#[tokio::test]
async fn test_mock_generate_url_link_error() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock error response
    Mock::given(method("POST"))
        .and(path("/wxa/generate_urllink"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errcode": 40125,
            "errmsg": "invalid appid"
        })))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let options = UrlLinkOptions {
        path: None,
        query: None,
        expire_type: None,
        expire_time: None,
        expire_interval: None,
    };

    let result = qrcode_api.generate_url_link(options).await;

    assert!(result.is_err());
}

/// Test successful generate_short_link with mock
#[tokio::test]
async fn test_mock_generate_short_link_success() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock the genwxashortlink endpoint
    Mock::given(method("POST"))
        .and(path("/wxa/genwxashortlink"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "link": "http://w.url.cn/s/XXXXXX",
            "errcode": 0,
            "errmsg": "ok"
        })))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let options = ShortLinkOptions {
        page_url: "https://example.com/page".to_string(),
    };

    let result = qrcode_api.generate_short_link(options).await;

    assert!(result.is_ok());
    let link = result.unwrap();
    assert!(link.starts_with("http://w.url.cn/s/"));
}

/// Test generate_short_link error response handling
#[tokio::test]
async fn test_mock_generate_short_link_error() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_qrcode",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock error response
    Mock::given(method("POST"))
        .and(path("/wxa/genwxashortlink"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errcode": 40213,
            "errmsg": "invalid page_url"
        })))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let options = ShortLinkOptions {
        page_url: "invalid_url".to_string(),
    };

    let result = qrcode_api.generate_short_link(options).await;

    assert!(result.is_err());
}

/// Test get_wxa_code_unlimit request body is correct
#[tokio::test]
async fn test_mock_get_wxa_code_unlimit_request_body() {
    let mock_server = MockServer::start().await;

    // Mock the token endpoint
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    // Mock that verifies the request body
    let expected_body = serde_json::json!({
        "scene": "test123",
        "page": "pages/index/index",
        "width": 430,
        "auto_color": false,
        "line_color": { "r": 255, "g": 0, "b": 0 },
        "is_hyaline": true
    });

    Mock::given(method("POST"))
        .and(path("/wxa/getwxacodeunlimit"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![
            0x89, 0x50, 0x4E, 0x47, // PNG header
        ]))
        .mount(&mock_server)
        .await;

    let context = create_test_context(&mock_server).await;
    let qrcode_api = QrcodeApi::new(context);

    let mut options = UnlimitQrcodeOptions::new("test123");
    options.page = Some("pages/index/index".to_string());
    options.width = Some(430);
    options.auto_color = Some(false);
    options.line_color = Some(LineColor { r: 255, g: 0, b: 0 });
    options.is_hyaline = Some(true);

    let result = qrcode_api.get_wxa_code_unlimit(options).await;

    // Should succeed because the request body matches
    assert!(result.is_ok());
}
