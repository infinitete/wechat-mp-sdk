//! Parity Baseline Tests - WechatMp vs api/* Implementations
//!
//! These tests verify that the facade methods in `WechatMp` produce
//! identical results to the direct `api/*` module implementations.
//!
//! ## Purpose
//!
//! This test suite establishes a baseline for the deduplication tasks (T13-T15).
//! Any failure indicates behavior drift between implementations.
//!
//! ## Test Groups
//!
//! 1. Template API: add_template, get_template_list, delete_template, get_category
//! 2. QRCode API: get_wxa_code, generate_url_scheme, generate_url_link
//! 3. User API: get_phone_number
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test parity_baseline_tests -- --nocapture
//! ```

use std::sync::Arc;

use wechat_mp_sdk::api::qrcode::{
    LineColor, QrcodeApi, QrcodeOptions, UrlLinkOptions, UrlSchemeOptions,
};
use wechat_mp_sdk::api::template::TemplateApi;
use wechat_mp_sdk::api::user::UserApi;
use wechat_mp_sdk::api::WechatContext;
use wechat_mp_sdk::client::WechatClient;
use wechat_mp_sdk::token::TokenManager;
use wechat_mp_sdk::types::{AppId, AppSecret};
use wechat_mp_sdk::WechatMp;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// =============================================================================
// Test Utilities
// =============================================================================

/// Creates a WechatClient pointing to the mock server
fn create_test_client(mock_server_uri: &str) -> WechatClient {
    let appid = AppId::new("wx1234567890abcdef").unwrap();
    let secret = AppSecret::new("test_secret_12345").unwrap();

    WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .base_url(mock_server_uri)
        .build()
        .unwrap()
}

/// Creates a WechatContext pointing to the mock server (for Api modules)
async fn create_test_context(mock_server: &MockServer) -> Arc<WechatContext> {
    let client = create_test_client(&mock_server.uri());
    let token_manager = TokenManager::new(client.clone());
    Arc::new(WechatContext::new(
        Arc::new(client),
        Arc::new(token_manager),
    ))
}

/// Creates a WechatMp instance pointing to the mock server (for facade)
async fn create_test_wechat_mp(mock_server: &MockServer) -> WechatMp {
    let appid = AppId::new("wx1234567890abcdef").unwrap();
    let secret = AppSecret::new("test_secret_12345").unwrap();

    WechatMp::builder()
        .appid(appid)
        .secret(secret)
        .base_url(mock_server.uri())
        .build()
        .unwrap()
}

/// Sets up the token mock endpoint
async fn mock_token_endpoint(mock_server: &MockServer, token: &str) {
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": token,
            "expires_in": 7200
        })))
        .mount(mock_server)
        .await;
}

// =============================================================================
// TEMPLATE API PARITY TESTS
// =============================================================================

mod template_api_parity {
    use super::*;

    /// PARITY TEST: get_template_list - success path
    ///
    /// Verifies that WechatMp::get_template_list() and TemplateApi::get_template_list()
    /// return identical results when given the same successful API response.
    #[tokio::test]
    async fn parity_get_template_list_success() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_template").await;

        let response_body = serde_json::json!({
            "data": [
                {
                    "priTmplId": "template_id_001",
                    "title": "Order Confirmation",
                    "content": "Order: {{thing1.DATA}}",
                    "type": 2
                },
                {
                    "priTmplId": "template_id_002",
                    "title": "Payment Notification",
                    "content": "Amount: {{amount1.DATA}}",
                    "example": "Example content",
                    "type": 2
                }
            ],
            "errcode": 0,
            "errmsg": "ok"
        });

        Mock::given(method("GET"))
            .and(path("/wxaapi/newtmpl/gettemplate"))
            .and(query_param("access_token", "test_token_template"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body.clone()))
            .mount(&mock_server)
            .await;

        // Create both implementations
        let context = create_test_context(&mock_server).await;
        let template_api = TemplateApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        // Call both - WechatMp facade
        let facade_result = wechat_mp.get_template_list().await;

        // Call both - TemplateApi
        let api_result = template_api.get_template_list().await;

        // PARITY CHECK: Both should succeed
        assert!(
            facade_result.is_ok(),
            "WechatMp::get_template_list should succeed"
        );
        assert!(
            api_result.is_ok(),
            "TemplateApi::get_template_list should succeed"
        );

        let facade_templates = facade_result.unwrap();
        let api_templates = api_result.unwrap();

        // PARITY CHECK: Same count
        assert_eq!(
            facade_templates.len(),
            api_templates.len(),
            "PARITY FAILURE: Template count mismatch"
        );

        // PARITY CHECK: Same content
        for (f, a) in facade_templates.iter().zip(api_templates.iter()) {
            assert_eq!(
                f.private_template_id, a.private_template_id,
                "PARITY FAILURE: private_template_id mismatch"
            );
            assert_eq!(f.title, a.title, "PARITY FAILURE: title mismatch");
            assert_eq!(f.content, a.content, "PARITY FAILURE: content mismatch");
        }
    }

    /// PARITY TEST: get_template_list - error path
    ///
    /// Verifies that both implementations handle API errors consistently.
    #[tokio::test]
    async fn parity_get_template_list_error() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_error").await;

        Mock::given(method("GET"))
            .and(path("/wxaapi/newtmpl/gettemplate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "errcode": 40001,
                "errmsg": "invalid credential"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let template_api = TemplateApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        // Call both implementations
        let facade_result = wechat_mp.get_template_list().await;
        let api_result = template_api.get_template_list().await;

        // PARITY CHECK: Both should fail
        assert!(
            facade_result.is_err(),
            "WechatMp::get_template_list should fail on API error"
        );
        assert!(
            api_result.is_err(),
            "TemplateApi::get_template_list should fail on API error"
        );

        // PARITY CHECK: Both should be Api errors
        use wechat_mp_sdk::WechatError;
        if let Err(WechatError::Api { code, .. }) = &facade_result {
            assert_eq!(*code, 40001, "Facade error code mismatch");
        } else {
            panic!("PARITY FAILURE: Facade should return Api error");
        }

        if let Err(WechatError::Api { code, .. }) = &api_result {
            assert_eq!(*code, 40001, "API error code mismatch");
        } else {
            panic!("PARITY FAILURE: Api should return Api error");
        }
    }

    /// PARITY TEST: add_template - success path
    #[tokio::test]
    async fn parity_add_template_success() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_add").await;

        Mock::given(method("POST"))
            .and(path("/wxaapi/newtmpl/addtemplate"))
            .and(query_param("access_token", "test_token_add"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "priTmplId": "new_private_template_id_123",
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let template_api = TemplateApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        // Call both implementations
        let facade_result = wechat_mp
            .add_template("AA1234", Some(vec![1, 2, 3]), Some("test scene"))
            .await;
        let api_result = template_api
            .add_template("AA1234", Some(vec![1, 2, 3]), Some("test scene"))
            .await;

        // PARITY CHECK: Both should succeed
        assert!(
            facade_result.is_ok(),
            "WechatMp::add_template should succeed"
        );
        assert!(
            api_result.is_ok(),
            "TemplateApi::add_template should succeed"
        );

        // PARITY CHECK: Same result
        assert_eq!(
            facade_result.unwrap(),
            api_result.unwrap(),
            "PARITY FAILURE: add_template result mismatch"
        );
    }

    /// PARITY TEST: delete_template - success path
    #[tokio::test]
    async fn parity_delete_template_success() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_del").await;

        Mock::given(method("POST"))
            .and(path("/wxaapi/newtmpl/deltemplate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let template_api = TemplateApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        // Call both implementations
        let facade_result = wechat_mp.delete_template("template_to_delete").await;
        let api_result = template_api.delete_template("template_to_delete").await;

        // PARITY CHECK: Both should succeed
        assert!(
            facade_result.is_ok(),
            "WechatMp::delete_template should succeed"
        );
        assert!(
            api_result.is_ok(),
            "TemplateApi::delete_template should succeed"
        );
    }

    /// PARITY TEST: get_category - success path
    #[tokio::test]
    async fn parity_get_category_success() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_cat").await;

        Mock::given(method("GET"))
            .and(path("/wxaapi/newtmpl/getcategory"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"id": 1, "name": "IT Technology"},
                    {"id": 2, "name": "E-commerce"}
                ],
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let template_api = TemplateApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        // Call both implementations
        let facade_result = wechat_mp.get_category().await;
        let api_result = template_api.get_category().await;

        // PARITY CHECK: Both should succeed
        assert!(
            facade_result.is_ok(),
            "WechatMp::get_category should succeed"
        );
        assert!(
            api_result.is_ok(),
            "TemplateApi::get_category should succeed"
        );

        let facade_cats = facade_result.unwrap();
        let api_cats = api_result.unwrap();

        // PARITY CHECK: Same content
        assert_eq!(facade_cats.len(), api_cats.len(), "Category count mismatch");
        for (f, a) in facade_cats.iter().zip(api_cats.iter()) {
            assert_eq!(f.id, a.id, "Category id mismatch");
            assert_eq!(f.name, a.name, "Category name mismatch");
        }
    }
}

// =============================================================================
// QRCODE API PARITY TESTS
// =============================================================================

mod qrcode_api_parity {
    use super::*;

    /// Minimal valid PNG for mock responses
    const VALID_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    /// PARITY TEST: get_wxa_code - success path (returns image)
    #[tokio::test]
    async fn parity_get_wxa_code_success() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_qrcode").await;

        Mock::given(method("POST"))
            .and(path("/wxa/getwxacode"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(VALID_PNG.to_vec())
                    .insert_header("content-type", "image/png"),
            )
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let qrcode_api = QrcodeApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        let mut options = QrcodeOptions::new();
        options.path = Some("/pages/index/index".to_string());
        options.width = Some(430);
        options.auto_color = Some(false);
        options.line_color = Some(LineColor { r: 0, g: 0, b: 0 });
        options.is_hyaline = Some(false);

        let facade_result = wechat_mp.get_wxa_code(options.clone()).await;
        let api_result = qrcode_api.get_wxa_code(options).await;

        assert!(
            facade_result.is_ok(),
            "WechatMp::get_wxa_code should succeed"
        );
        assert!(api_result.is_ok(), "QrcodeApi::get_wxa_code should succeed");

        let facade_bytes = facade_result.unwrap();
        let api_bytes = api_result.unwrap();

        // PARITY CHECK: Same bytes returned
        assert_eq!(
            facade_bytes, api_bytes,
            "PARITY FAILURE: QR code bytes mismatch"
        );

        // Verify PNG header
        assert!(
            facade_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
            "Result should be PNG"
        );
    }

    /// PARITY TEST: get_wxa_code - error path (API returns JSON error)
    #[tokio::test]
    async fn parity_get_wxa_code_error() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_qrcode_err").await;

        Mock::given(method("POST"))
            .and(path("/wxa/getwxacode"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({
                        "errcode": 45009,
                        "errmsg": "Frequency limit reached"
                    }))
                    .insert_header("content-type", "application/json"),
            )
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let qrcode_api = QrcodeApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        let mut options = QrcodeOptions::new();
        options.path = Some("/pages/index/index".to_string());

        let facade_result = wechat_mp.get_wxa_code(options.clone()).await;
        let api_result = qrcode_api.get_wxa_code(options).await;
        assert!(
            facade_result.is_err(),
            "WechatMp::get_wxa_code should fail on API error"
        );
        assert!(
            api_result.is_err(),
            "QrcodeApi::get_wxa_code should fail on API error"
        );

        // PARITY CHECK: Both should be Api errors with same code
        use wechat_mp_sdk::WechatError;
        if let Err(WechatError::Api { code: f_code, .. }) = &facade_result {
            if let Err(WechatError::Api { code: a_code, .. }) = &api_result {
                assert_eq!(f_code, a_code, "PARITY FAILURE: Error codes should match");
                assert_eq!(*f_code, 45009, "Error code should be 45009");
            } else {
                panic!("PARITY FAILURE: API result should be Api error");
            }
        } else {
            panic!("PARITY FAILURE: Facade result should be Api error");
        }
    }

    /// PARITY TEST: generate_url_scheme - success path
    #[tokio::test]
    async fn parity_generate_url_scheme_success() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_scheme").await;

        Mock::given(method("POST"))
            .and(path("/wxa/generatescheme"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "openlink": "weixin://dl/business/?t=XXXXXXXXXX",
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let qrcode_api = QrcodeApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        let options = UrlSchemeOptions {
            path: Some("/pages/index/index".to_string()),
            query: Some("id=123".to_string()),
            expire: None,
        };

        // Call both implementations
        let facade_result = wechat_mp.generate_url_scheme(options.clone()).await;
        let api_result = qrcode_api.generate_url_scheme(options).await;

        // PARITY CHECK: Both should succeed
        assert!(
            facade_result.is_ok(),
            "WechatMp::generate_url_scheme should succeed"
        );
        assert!(
            api_result.is_ok(),
            "QrcodeApi::generate_url_scheme should succeed"
        );

        // PARITY CHECK: Same result
        assert_eq!(
            facade_result.unwrap(),
            api_result.unwrap(),
            "PARITY FAILURE: URL scheme mismatch"
        );
    }

    /// PARITY TEST: generate_url_link - success path
    #[tokio::test]
    async fn parity_generate_url_link_success() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_link").await;

        Mock::given(method("POST"))
            .and(path("/wxa/generate_urllink"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "link": "https://wxaurl.cn/XXXXXXXXXX",
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let qrcode_api = QrcodeApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        let options = UrlLinkOptions {
            path: Some("/pages/index/index".to_string()),
            query: Some("id=456".to_string()),
            expire_type: Some(1),
            expire_time: Some(1672531200),
            expire_interval: None,
        };

        // Call both implementations
        let facade_result = wechat_mp.generate_url_link(options.clone()).await;
        let api_result = qrcode_api.generate_url_link(options).await;

        // PARITY CHECK: Both should succeed
        assert!(
            facade_result.is_ok(),
            "WechatMp::generate_url_link should succeed"
        );
        assert!(
            api_result.is_ok(),
            "QrcodeApi::generate_url_link should succeed"
        );

        // PARITY CHECK: Same result
        assert_eq!(
            facade_result.unwrap(),
            api_result.unwrap(),
            "PARITY FAILURE: URL link mismatch"
        );
    }

    /// PARITY TEST: generate_url_scheme - error path
    #[tokio::test]
    async fn parity_generate_url_scheme_error() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_scheme_err").await;

        Mock::given(method("POST"))
            .and(path("/wxa/generatescheme"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "openlink": "",
                "errcode": 40097,
                "errmsg": "invalid page path"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let qrcode_api = QrcodeApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        let options = UrlSchemeOptions {
            path: Some("/invalid/path".to_string()),
            query: None,
            expire: None,
        };

        // Call both implementations
        let facade_result = wechat_mp.generate_url_scheme(options.clone()).await;
        let api_result = qrcode_api.generate_url_scheme(options).await;

        // PARITY CHECK: Both should fail
        assert!(
            facade_result.is_err(),
            "WechatMp::generate_url_scheme should fail"
        );
        assert!(
            api_result.is_err(),
            "QrcodeApi::generate_url_scheme should fail"
        );
    }
}

// =============================================================================
// USER API PARITY TESTS
// =============================================================================

mod user_api_parity {
    use super::*;

    /// PARITY TEST: get_phone_number - success path
    #[tokio::test]
    async fn parity_get_phone_number_success() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_phone").await;

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
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let user_api = UserApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        // Call both implementations
        let facade_result = wechat_mp.get_phone_number("phone_code_123").await;
        let api_result = user_api.get_phone_number("phone_code_123").await;

        // PARITY CHECK: Both should succeed
        assert!(
            facade_result.is_ok(),
            "WechatMp::get_phone_number should succeed"
        );
        assert!(
            api_result.is_ok(),
            "UserApi::get_phone_number should succeed"
        );

        let facade_response = facade_result.unwrap();
        let api_response = api_result.unwrap();

        // PARITY CHECK: Same phone info
        assert_eq!(
            facade_response.phone_info.phone_number, api_response.phone_info.phone_number,
            "PARITY FAILURE: phone_number mismatch"
        );
        assert_eq!(
            facade_response.phone_info.pure_phone_number, api_response.phone_info.pure_phone_number,
            "PARITY FAILURE: pure_phone_number mismatch"
        );
        assert_eq!(
            facade_response.phone_info.country_code, api_response.phone_info.country_code,
            "PARITY FAILURE: country_code mismatch"
        );
    }

    /// PARITY TEST: get_phone_number - error path
    #[tokio::test]
    async fn parity_get_phone_number_error() {
        let mock_server = MockServer::start().await;

        mock_token_endpoint(&mock_server, "test_token_phone_err").await;

        Mock::given(method("POST"))
            .and(path("/wxa/business/getuserphonenumber"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "phone_info": {
                    "phone_number": "",
                    "pure_phone_number": "",
                    "country_code": "",
                    "watermark": {
                        "timestamp": 0,
                        "appid": ""
                    }
                },
                "errcode": 40001,
                "errmsg": "invalid credential"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server).await;
        let user_api = UserApi::new(context);

        let wechat_mp = create_test_wechat_mp(&mock_server).await;

        // Call both implementations
        let facade_result = wechat_mp.get_phone_number("invalid_code").await;
        let api_result = user_api.get_phone_number("invalid_code").await;

        // PARITY CHECK: Both should fail
        assert!(
            facade_result.is_err(),
            "WechatMp::get_phone_number should fail on API error"
        );
        assert!(
            api_result.is_err(),
            "UserApi::get_phone_number should fail on API error"
        );

        // PARITY CHECK: Both should be Api errors
        use wechat_mp_sdk::WechatError;
        if let Err(WechatError::Api { code: f_code, .. }) = &facade_result {
            if let Err(WechatError::Api { code: a_code, .. }) = &api_result {
                assert_eq!(f_code, a_code, "PARITY FAILURE: Error codes should match");
                assert_eq!(*f_code, 40001, "Error code should be 40001");
            } else {
                panic!("PARITY FAILURE: API result should be Api error");
            }
        } else {
            panic!("PARITY FAILURE: Facade result should be Api error");
        }
    }
}

// =============================================================================
// PARITY STATUS SUMMARY
// =============================================================================

mod parity_summary {
    //! ## Current Parity Status
    //!
    //! All tests in this module should pass, indicating full behavioral parity
    //! between `WechatMp` facade methods and direct `api/*` module calls.
    //!
    //! ### Tested Endpoints
    //!
    //! | Endpoint Group | Facade Method | API Module | Status |
    //! |----------------|---------------|------------|--------|
    //! | Template | `get_template_list` | `TemplateApi::get_template_list` | ✅ Parity |
    //! | Template | `add_template` | `TemplateApi::add_template` | ✅ Parity |
    //! | Template | `delete_template` | `TemplateApi::delete_template` | ✅ Parity |
    //! | Template | `get_category` | `TemplateApi::get_category` | ✅ Parity |
    //! | QRCode | `get_wxa_code` | `QrcodeApi::get_wxa_code` | ✅ Parity |
    //! | QRCode | `generate_url_scheme` | `QrcodeApi::generate_url_scheme` | ✅ Parity |
    //! | QRCode | `generate_url_link` | `QrcodeApi::generate_url_link` | ✅ Parity |
    //! | User | `get_phone_number` | `UserApi::get_phone_number` | ✅ Parity |
    //!
    //! ### Known Differences
    //!
    //! None at this time. All tested endpoints show full parity.
    //!
    //! ### Implications for Deduplication (T13-T15)
    //!
    //! Since all endpoints demonstrate parity, the deduplication tasks can safely
    //! proceed by having `WechatMp` delegate to the `api/*` modules.
    //! This test suite serves as the regression guard.
}

mod inventory_baseline {
    use std::collections::{HashMap, HashSet};

    use wechat_mp_sdk::api::endpoint_inventory::get_endpoint_inventory;

    #[test]
    fn inventory_has_no_duplicate_ids() {
        let mut seen = HashSet::new();

        for item in get_endpoint_inventory() {
            assert!(
                seen.insert(item.endpoint_id),
                "duplicate endpoint_id detected: {}",
                item.endpoint_id
            );
        }
    }

    #[test]
    fn inventory_has_no_empty_categories() {
        let mut category_counts: HashMap<&str, usize> = HashMap::new();

        for item in get_endpoint_inventory() {
            *category_counts.entry(item.category).or_insert(0) += 1;
        }

        assert!(
            !category_counts.is_empty(),
            "inventory categories must not be empty"
        );

        for (category, count) in category_counts {
            assert!(
                count > 0,
                "category {} must have at least one endpoint",
                category
            );
        }
    }

    #[test]
    fn inventory_is_consistent() {
        let inventory = get_endpoint_inventory();
        assert!(!inventory.is_empty(), "inventory must not be empty");

        for item in inventory {
            assert!(
                !item.category.trim().is_empty(),
                "category must not be empty for {}",
                item.endpoint_id
            );
            assert!(
                !item.endpoint_id.trim().is_empty(),
                "endpoint_id must not be empty"
            );
            assert!(
                !item.http_method.trim().is_empty(),
                "http_method must not be empty for {}",
                item.endpoint_id
            );
            assert!(
                item.http_method == "GET" || item.http_method == "POST",
                "http_method must be GET or POST for {}",
                item.endpoint_id
            );
            assert!(
                !item.path.trim().is_empty(),
                "path must not be empty for {}",
                item.endpoint_id
            );
            assert!(
                item.path.starts_with('/'),
                "path must start with '/' for {}",
                item.endpoint_id
            );
        }
    }
}

mod coverage_matrix {
    use wechat_mp_sdk::api::endpoint_inventory::get_endpoint_inventory;

    #[derive(Debug, Clone, Copy)]
    struct CoverageStats {
        total_endpoints: usize,
        implemented: usize,
        missing_non_deprecated: usize,
        deprecated: usize,
    }

    fn collect_coverage_stats() -> CoverageStats {
        let inventory = get_endpoint_inventory();
        let total_endpoints = inventory.len();
        let implemented = inventory.iter().filter(|item| item.implemented).count();
        let deprecated = inventory.iter().filter(|item| item.deprecated).count();
        let missing_non_deprecated = inventory
            .iter()
            .filter(|item| !item.deprecated && !item.implemented)
            .count();

        CoverageStats {
            total_endpoints,
            implemented,
            missing_non_deprecated,
            deprecated,
        }
    }

    fn coverage_percent(stats: CoverageStats) -> f64 {
        let active_total = stats.total_endpoints.saturating_sub(stats.deprecated);
        if active_total == 0 {
            return 100.0;
        }

        let implemented_active = stats.implemented.saturating_sub(stats.deprecated);
        (implemented_active as f64 / active_total as f64) * 100.0
    }

    #[test]
    fn coverage_report_prints_summary() {
        let stats = collect_coverage_stats();
        let percent = coverage_percent(stats);

        println!("COVERAGE REPORT:");
        println!("total_endpoints={}", stats.total_endpoints);
        println!("implemented={}", stats.implemented);
        println!("missing_non_deprecated={}", stats.missing_non_deprecated);
        println!("deprecated={}", stats.deprecated);
        println!("coverage_percent={percent:.2}%");

        assert!(
            stats.total_endpoints > 0,
            "endpoint inventory must not be empty"
        );
    }

    #[test]
    #[ignore = "Enable when all non-deprecated endpoints are implemented"]
    fn non_deprecated_missing_endpoints_zero() {
        let stats = collect_coverage_stats();
        assert_eq!(
            stats.missing_non_deprecated, 0,
            "non-deprecated endpoint coverage must be 100%"
        );
    }

    #[test]
    fn fails_when_endpoint_unmapped() {
        let stats = collect_coverage_stats();
        assert!(
            stats.missing_non_deprecated > 0,
            "expected at least one non-deprecated endpoint to be unmapped; if this fails, remove #[ignore] from non_deprecated_missing_endpoints_zero"
        );
    }
}

// =============================================================================
// FACADE GUARD TESTS
// =============================================================================

mod facade_guards {
    //! Guard tests that verify the WechatMp facade stays in sync with the
    //! endpoint inventory. These tests detect when a new API module method
    //! exists (implemented: true) but has no corresponding facade method.

    use std::collections::{HashMap, HashSet};

    use wechat_mp_sdk::api::endpoint_inventory::get_endpoint_inventory;

    /// Maps endpoint_id -> facade method name for all currently-implemented endpoints.
    ///
    /// MAINTENANCE: When you set `implemented: true` for a new endpoint in
    /// `endpoint_inventory.rs`, you MUST add a corresponding entry here AND
    /// add the facade method to `WechatMp` in `src/client/wechat_mp.rs`.
    const FACADE_METHOD_MAP: &[(&str, &str)] = &[
        ("accessToken.getAccessToken", "get_access_token"),
        ("auth.code2Session", "auth_login"),
        ("user.getPhoneNumber", "get_phone_number"),
        ("qrcode.getQRCode", "get_wxa_code"),
        ("qrcode.getUnlimitedQRCode", "get_wxa_code_unlimit"),
        ("qrcode.createQRCode", "create_qrcode"),
        ("qrcode.generateScheme", "generate_url_scheme"),
        ("qrcode.generateUrlLink", "generate_url_link"),
        ("qrcode.generateShortLink", "generate_short_link"),
        (
            "customerService.sendCustomMessage",
            "send_customer_service_message",
        ),
        ("customerService.uploadTempMedia", "upload_temp_media"),
        ("customerService.getTempMedia", "get_temp_media"),
        ("subscribe.sendMessage", "send_subscribe_message"),
        ("subscribe.addMessageTemplate", "add_template"),
        ("subscribe.deleteMessageTemplate", "delete_template"),
        ("subscribe.getCategory", "get_category"),
        ("subscribe.getMessageTemplateList", "get_template_list"),
    ];

    /// Verifies that every currently-implemented endpoint in the inventory has
    /// a corresponding entry in FACADE_METHOD_MAP.
    ///
    /// If this test fails, it means a new endpoint was marked `implemented: true`
    /// in the inventory but no facade method mapping was added here.
    #[test]
    fn facade_has_all_currently_implemented_methods() {
        let inventory = get_endpoint_inventory();

        // Build a set of endpoint_ids that are implemented (non-deprecated)
        let implemented_ids: HashSet<&str> = inventory
            .iter()
            .filter(|item| item.implemented && !item.deprecated)
            .map(|item| item.endpoint_id)
            .collect();

        // Build a set of endpoint_ids covered by the facade map
        let facade_covered_ids: HashSet<&str> = FACADE_METHOD_MAP
            .iter()
            .map(|(endpoint_id, _)| *endpoint_id)
            .collect();

        // Every implemented endpoint must have a facade mapping
        let mut missing_from_facade: Vec<&str> = implemented_ids
            .difference(&facade_covered_ids)
            .copied()
            .collect();

        if !missing_from_facade.is_empty() {
            missing_from_facade.sort();
            panic!(
                "GUARD FAILURE: {} implemented endpoint(s) have no facade method mapping:\n  - {}\n\nAdd entries to FACADE_METHOD_MAP in tests/parity_baseline_tests.rs\nAND add the corresponding facade method to WechatMp in src/client/wechat_mp.rs",
                missing_from_facade.len(),
                missing_from_facade.join("\n  - "),
            );
        }
    }

    /// Verifies that FACADE_METHOD_MAP doesn't reference endpoint_ids that
    /// don't exist in the inventory (prevents stale entries).
    #[test]
    fn facade_map_has_no_phantom_endpoints() {
        let inventory = get_endpoint_inventory();

        let all_inventory_ids: HashSet<&str> =
            inventory.iter().map(|item| item.endpoint_id).collect();

        let mut phantom_entries = Vec::new();
        for (endpoint_id, facade_method) in FACADE_METHOD_MAP {
            if !all_inventory_ids.contains(endpoint_id) {
                phantom_entries.push(format!("{} -> {}", endpoint_id, facade_method));
            }
        }

        assert!(
            phantom_entries.is_empty(),
            "GUARD FAILURE: FACADE_METHOD_MAP references endpoint_ids not in inventory:\n  - {}\n",
            phantom_entries.join("\n  - ")
        );
    }

    /// Verifies that FACADE_METHOD_MAP has no duplicate endpoint_id entries.
    #[test]
    fn facade_map_has_no_duplicate_entries() {
        let mut seen: HashMap<&str, &str> = HashMap::new();
        let mut duplicates = Vec::new();

        for (endpoint_id, facade_method) in FACADE_METHOD_MAP {
            if let Some(existing) = seen.insert(endpoint_id, facade_method) {
                duplicates.push(format!(
                    "'{}' appears twice: '{}' and '{}'",
                    endpoint_id, existing, facade_method
                ));
            }
        }

        assert!(
            duplicates.is_empty(),
            "GUARD FAILURE: FACADE_METHOD_MAP has duplicate entries:\n  - {}\n",
            duplicates.join("\n  - ")
        );
    }

    /// Compile-time guard: verifies existing facade methods still exist on WechatMp.
    ///
    /// If this test fails to compile, a facade method was removed or renamed.
    /// Taking a reference to a method is sufficient to verify it exists at compile time.
    #[test]
    fn existing_facade_methods_are_backward_compatible() {
        use wechat_mp_sdk::WechatMp;

        // Verify methods exist by taking unambiguous references to them.
        // This is a compile-time check - if any method is removed or renamed,
        // this test will fail to compile.
        let _methods: &[*const ()] = &[
            WechatMp::get_access_token as *const (),
            WechatMp::auth_login as *const (),
            WechatMp::get_phone_number as *const (),
            WechatMp::send_customer_service_message as *const (),
            WechatMp::get_template_list as *const (),
            WechatMp::add_template as *const (),
            WechatMp::delete_template as *const (),
            WechatMp::get_category as *const (),
            WechatMp::send_subscribe_message as *const (),
            WechatMp::get_wxa_code as *const (),
            WechatMp::get_wxa_code_unlimit as *const (),
            WechatMp::create_qrcode as *const (),
            WechatMp::generate_url_scheme as *const (),
            WechatMp::generate_url_link as *const (),
            WechatMp::generate_short_link as *const (),
            WechatMp::upload_temp_media as *const (),
            WechatMp::get_temp_media as *const (),
            WechatMp::invalidate_token as *const (),
        ];

        // Verify the sync method with a typed function pointer (no lifetime issues)
        let _: fn(&WechatMp) -> &str = WechatMp::appid;

        // If we reach here, all methods exist with compatible signatures
        println!(
            "All {} facade methods verified at compile time",
            FACADE_METHOD_MAP.len()
        );
    }
}
