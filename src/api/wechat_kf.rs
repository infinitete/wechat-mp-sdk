//! WeChat Customer Service (KF) API
//!
//! Endpoints for binding and managing WeChat open customer service accounts.
//!
//! # Endpoints
//!
//! - [`WechatKfApi::get_kf_work_bound`] - Get bound open KF account IDs
//! - [`WechatKfApi::bind_kf_work`] - Bind an open KF account
//! - [`WechatKfApi::unbind_kf_work`] - Unbind an open KF account

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

// ============================================================================
// Request Types (internal)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct GetKfWorkBoundRequest {
    openid: String,
}

#[derive(Debug, Clone, Serialize)]
struct BindKfWorkRequest {
    openid: String,
    open_kfid: String,
}

#[derive(Debug, Clone, Serialize)]
struct UnbindKfWorkRequest {
    openid: String,
    open_kfid: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BaseApiResponse {
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

// ============================================================================
// Public Response Types
// ============================================================================

/// KF work info entry
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct KfWorkInfo {
    /// Open KF account ID
    #[serde(default)]
    pub open_kfid: String,
    /// KF account name
    #[serde(default)]
    pub kf_name: String,
}

/// Response from getKfWorkBound
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KfWorkBoundResponse {
    /// List of bound KF accounts
    #[serde(default)]
    pub kf_list: Vec<KfWorkInfo>,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

// ============================================================================
// WechatKfApi
// ============================================================================

/// WeChat Customer Service (KF) API
///
/// Provides methods for binding and managing WeChat open customer service accounts.
pub struct WechatKfApi {
    context: Arc<WechatContext>,
}

impl WechatKfApi {
    /// Create a new WechatKfApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Get bound open KF account IDs for a user
    ///
    /// POST /cgi-bin/kfaccount/getbindedopenkfid?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `openid` - User's OpenID
    pub async fn get_kf_work_bound(
        &self,
        openid: &str,
    ) -> Result<KfWorkBoundResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!(
            "/cgi-bin/kfaccount/getbindedopenkfid?access_token={}",
            access_token
        );
        let body = GetKfWorkBoundRequest {
            openid: openid.to_string(),
        };
        let response: KfWorkBoundResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Bind an open KF account to a user
    ///
    /// POST /cgi-bin/kfaccount/bindopenkfid?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `openid` - User's OpenID
    /// * `open_kfid` - Open KF account ID to bind
    pub async fn bind_kf_work(&self, openid: &str, open_kfid: &str) -> Result<(), WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!(
            "/cgi-bin/kfaccount/bindopenkfid?access_token={}",
            access_token
        );
        let body = BindKfWorkRequest {
            openid: openid.to_string(),
            open_kfid: open_kfid.to_string(),
        };
        let response: BaseApiResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(())
    }

    /// Unbind an open KF account from a user
    ///
    /// POST /cgi-bin/kfaccount/unbindopenkfid?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `openid` - User's OpenID
    /// * `open_kfid` - Open KF account ID to unbind
    pub async fn unbind_kf_work(&self, openid: &str, open_kfid: &str) -> Result<(), WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!(
            "/cgi-bin/kfaccount/unbindopenkfid?access_token={}",
            access_token
        );
        let body = UnbindKfWorkRequest {
            openid: openid.to_string(),
            open_kfid: open_kfid.to_string(),
        };
        let response: BaseApiResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(())
    }
}

impl WechatApi for WechatKfApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "wechat_kf"
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
    fn test_kf_work_bound_response_parse() {
        let json = r#"{
            "kf_list": [
                {
                    "open_kfid": "kf_abc123",
                    "kf_name": "Customer Support"
                },
                {
                    "open_kfid": "kf_def456",
                    "kf_name": "Sales"
                }
            ],
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: KfWorkBoundResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.kf_list.len(), 2);
        assert_eq!(response.kf_list[0].open_kfid, "kf_abc123");
        assert_eq!(response.kf_list[0].kf_name, "Customer Support");
        assert_eq!(response.kf_list[1].open_kfid, "kf_def456");
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_kf_work_bound_response_defaults() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: KfWorkBoundResponse = serde_json::from_str(json).unwrap();
        assert!(response.kf_list.is_empty());
    }

    #[test]
    fn test_kf_work_info_defaults() {
        let json = r#"{}"#;
        let info: KfWorkInfo = serde_json::from_str(json).unwrap();
        assert!(info.open_kfid.is_empty());
        assert!(info.kf_name.is_empty());
    }

    #[test]
    fn test_api_name() {
        let context = create_test_context("http://localhost:0");
        let api = WechatKfApi::new(context);
        assert_eq!(api.api_name(), "wechat_kf");
    }

    // ---- Wiremock Integration Tests ----

    #[tokio::test]
    async fn test_get_kf_work_bound_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/kfaccount/getbindedopenkfid"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "kf_list": [{"open_kfid": "kf_abc", "kf_name": "Support"}],
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = WechatKfApi::new(context);
        let result = api.get_kf_work_bound("openid123").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.kf_list.len(), 1);
        assert_eq!(response.kf_list[0].open_kfid, "kf_abc");
    }

    #[tokio::test]
    async fn test_get_kf_work_bound_api_error() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/kfaccount/getbindedopenkfid"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 40001,
                "errmsg": "invalid credential"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = WechatKfApi::new(context);
        let result = api.get_kf_work_bound("openid123").await;
        assert!(result.is_err());
        if let Err(WechatError::Api { code, message }) = result {
            assert_eq!(code, 40001);
            assert_eq!(message, "invalid credential");
        } else {
            panic!("Expected WechatError::Api");
        }
    }

    #[tokio::test]
    async fn test_bind_kf_work_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/kfaccount/bindopenkfid"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = WechatKfApi::new(context);
        let result = api.bind_kf_work("openid123", "kf_abc").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unbind_kf_work_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/kfaccount/unbindopenkfid"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = WechatKfApi::new(context);
        let result = api.unbind_kf_work("openid123", "kf_abc").await;
        assert!(result.is_ok());
    }
}
