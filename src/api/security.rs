//! Security API
//!
//! Endpoints for content security checks including text, media, and user risk assessment.
//!
//! # Endpoints
//!
//! - [`SecurityApi::msg_sec_check`] - Check text content for policy violations
//! - [`SecurityApi::media_check_async`] - Async check media for policy violations
//! - [`SecurityApi::get_user_risk_rank`] - Get user risk rank score

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

// ============================================================================
// Request Types (internal)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct MsgSecCheckRequest {
    version: u8,
    openid: String,
    scene: u8,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct MediaCheckAsyncRequest {
    media_url: String,
    media_type: u8,
    version: u8,
    openid: String,
    scene: u8,
}

#[derive(Debug, Clone, Serialize)]
struct UserRiskRankRequest {
    appid: String,
    openid: String,
    scene: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mobile_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extended_info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_test: Option<bool>,
}

// ============================================================================
// Public Response Types
// ============================================================================

/// Detail item from message security check
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MsgSecCheckDetail {
    /// Strategy used
    #[serde(default)]
    pub strategy: String,
    /// Error code for this detail
    #[serde(default)]
    pub errcode: i32,
    /// Suggestion: "pass", "risky", or "review"
    #[serde(default)]
    pub suggest: String,
    /// Label classification (100=normal, 10001=ad, etc.)
    #[serde(default)]
    pub label: i32,
    /// Matched keyword (if any)
    #[serde(default)]
    pub keyword: String,
    /// Confidence probability (0-100)
    #[serde(default)]
    pub prob: i32,
}

/// Result summary from message security check
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MsgSecCheckResult {
    /// Suggestion: "pass", "risky", or "review"
    #[serde(default)]
    pub suggest: String,
    /// Label classification
    #[serde(default)]
    pub label: i32,
}

/// Response from msgSecCheck
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MsgSecCheckResponse {
    /// Overall result
    #[serde(default)]
    pub result: MsgSecCheckResult,
    /// Detailed results per strategy
    #[serde(default)]
    pub detail: Vec<MsgSecCheckDetail>,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Response from mediaCheckAsync
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MediaCheckAsyncResponse {
    /// Trace ID for querying result
    #[serde(default)]
    pub trace_id: String,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Response from getUserRiskRank
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserRiskRankResponse {
    /// Risk rank: 0-4 (0=no risk, 4=highest risk)
    #[serde(default)]
    pub risk_rank: i32,
    /// Union ID (note: WeChat API field is "unoin_id", not "union_id")
    #[serde(default)]
    pub unoin_id: i32,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Options for getUserRiskRank
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct UserRiskRankOptions {
    /// Client IP address
    pub client_ip: Option<String>,
    /// Mobile number
    pub mobile_no: Option<String>,
    /// Email address
    pub email_address: Option<String>,
    /// Extended info string
    pub extended_info: Option<String>,
    /// Whether this is a test request
    pub is_test: Option<bool>,
}

// ============================================================================
// SecurityApi
// ============================================================================

/// Security API
///
/// Provides methods for content security checks including text, media,
/// and user risk assessment.
pub struct SecurityApi {
    context: Arc<WechatContext>,
}

impl SecurityApi {
    /// Create a new SecurityApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Check text content for policy violations
    ///
    /// POST /wxa/msg_sec_check?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `openid` - User's OpenID
    /// * `scene` - Scene value (1=profile, 2=comment, 3=forum, 4=social log)
    /// * `content` - Text content to check
    pub async fn msg_sec_check(
        &self,
        openid: &str,
        scene: u8,
        content: &str,
    ) -> Result<MsgSecCheckResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(
            "/wxa/msg_sec_check?access_token={}",
            &access_token,
        );
        let body = MsgSecCheckRequest {
            version: 2,
            openid: openid.to_string(),
            scene,
            content: content.to_string(),
        };
        let response: MsgSecCheckResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Async check media (image/audio) for policy violations
    ///
    /// POST /wxa/media_check_async?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `media_url` - URL of the media to check
    /// * `media_type` - Media type (1=audio, 2=image)
    /// * `openid` - User's OpenID
    /// * `scene` - Scene value
    pub async fn media_check_async(
        &self,
        media_url: &str,
        media_type: u8,
        openid: &str,
        scene: u8,
    ) -> Result<MediaCheckAsyncResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(
            "/wxa/media_check_async?access_token={}",
            &access_token,
        );
        let body = MediaCheckAsyncRequest {
            media_url: media_url.to_string(),
            media_type,
            version: 2,
            openid: openid.to_string(),
            scene,
        };
        let response: MediaCheckAsyncResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Get user risk rank score
    ///
    /// POST /wxa/getuserriskrank?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `openid` - User's OpenID
    /// * `scene` - Scene value (0=registration, 1=marketing)
    /// * `options` - Additional optional parameters
    pub async fn get_user_risk_rank(
        &self,
        openid: &str,
        scene: u8,
        options: Option<UserRiskRankOptions>,
    ) -> Result<UserRiskRankResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(
            "/wxa/getuserriskrank?access_token={}",
            &access_token,
        );
        let opts = options.unwrap_or_default();
        let body = UserRiskRankRequest {
            appid: self.context.client.appid().to_string(),
            openid: openid.to_string(),
            scene,
            client_ip: opts.client_ip,
            mobile_no: opts.mobile_no,
            email_address: opts.email_address,
            extended_info: opts.extended_info,
            is_test: opts.is_test,
        };
        let response: UserRiskRankResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for SecurityApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "security"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::WechatClient;
    use crate::token::TokenManager;
    use crate::types::{AppId, AppSecret};

    fn create_test_context(base_url: &str) -> Arc<WechatContext> {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        let client = Arc::new(
            WechatClient::builder()
                .appid(appid)
                .secret(secret)
                .base_url(base_url)
                .build()
                .unwrap(),
        );
        let token_manager = Arc::new(TokenManager::new((*client).clone()));
        Arc::new(WechatContext::new(client, token_manager))
    }

    async fn setup_token_mock(mock_server: &wiremock::MockServer) {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, ResponseTemplate};

        Mock::given(method("GET"))
            .and(path("/cgi-bin/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "test_token",
                "expires_in": 7200,
                "errcode": 0,
                "errmsg": ""
            })))
            .mount(mock_server)
            .await;
    }

    // ---- Deserialization Tests ----

    #[test]
    fn test_msg_sec_check_response_parse() {
        let json = r#"{
            "result": {
                "suggest": "pass",
                "label": 100
            },
            "detail": [
                {
                    "strategy": "content_model",
                    "errcode": 0,
                    "suggest": "pass",
                    "label": 100,
                    "keyword": "",
                    "prob": 90
                }
            ],
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: MsgSecCheckResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.result.suggest, "pass");
        assert_eq!(response.result.label, 100);
        assert_eq!(response.detail.len(), 1);
        assert_eq!(response.detail[0].strategy, "content_model");
        assert_eq!(response.detail[0].prob, 90);
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_msg_sec_check_response_defaults() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: MsgSecCheckResponse = serde_json::from_str(json).unwrap();
        assert!(response.detail.is_empty());
        assert!(response.result.suggest.is_empty());
    }

    #[test]
    fn test_media_check_async_response_parse() {
        let json = r#"{
            "trace_id": "trace_abc123",
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: MediaCheckAsyncResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.trace_id, "trace_abc123");
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_user_risk_rank_response_parse() {
        let json = r#"{
            "risk_rank": 2,
            "unoin_id": 12345,
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: UserRiskRankResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.risk_rank, 2);
        assert_eq!(response.unoin_id, 12345);
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_user_risk_rank_response_defaults() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: UserRiskRankResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.risk_rank, 0);
        assert_eq!(response.unoin_id, 0);
    }

    #[test]
    fn test_api_name() {
        let context = create_test_context("http://localhost:0");
        let api = SecurityApi::new(context);
        assert_eq!(api.api_name(), "security");
    }

    // ---- Wiremock Integration Tests ----

    #[tokio::test]
    async fn test_msg_sec_check_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/wxa/msg_sec_check"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": {"suggest": "pass", "label": 100},
                "detail": [{"strategy": "content_model", "errcode": 0, "suggest": "pass", "label": 100, "keyword": "", "prob": 90}],
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = SecurityApi::new(context);
        let result = api.msg_sec_check("openid123", 1, "hello world").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.result.suggest, "pass");
    }

    #[tokio::test]
    async fn test_msg_sec_check_api_error() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/wxa/msg_sec_check"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 87014,
                "errmsg": "risky content"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = SecurityApi::new(context);
        let result = api.msg_sec_check("openid123", 1, "bad content").await;
        assert!(result.is_err());
        if let Err(WechatError::Api { code, message }) = result {
            assert_eq!(code, 87014);
            assert_eq!(message, "risky content");
        } else {
            panic!("Expected WechatError::Api");
        }
    }

    #[tokio::test]
    async fn test_media_check_async_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/wxa/media_check_async"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "trace_id": "trace_123",
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = SecurityApi::new(context);
        let result = api
            .media_check_async("https://example.com/image.jpg", 2, "openid123", 1)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trace_id, "trace_123");
    }

    #[tokio::test]
    async fn test_get_user_risk_rank_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/wxa/getuserriskrank"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "risk_rank": 1,
                "unoin_id": 99,
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = SecurityApi::new(context);
        let result = api.get_user_risk_rank("openid123", 0, None).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.risk_rank, 1);
        assert_eq!(response.unoin_id, 99);
    }
}
