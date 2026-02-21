//! OpenAPI Management API
//!
//! Endpoints for managing API quotas, rate limits, and diagnostic information.
//!
//! # Endpoints
//!
//! - [`OpenApiApi::clear_quota`] - Reset all API call quotas
//! - [`OpenApiApi::get_api_quota`] - Query API call quota for a specific endpoint
//! - [`OpenApiApi::clear_api_quota`] - Reset quota for a specific endpoint
//! - [`OpenApiApi::clear_quota_by_app_secret`] - Reset quota using AppSecret (no token)
//! - [`OpenApiApi::get_rid_info`] - Get request debug information by rid
//! - [`OpenApiApi::callback_check`] - Check callback URL connectivity
//! - [`OpenApiApi::get_api_domain_ip`] - Get WeChat API server IP addresses
//! - [`OpenApiApi::get_callback_ip`] - Get WeChat callback server IP addresses

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

// ============================================================================
// Request Types (internal)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct ClearQuotaRequest {
    appid: String,
}

#[derive(Debug, Clone, Serialize)]
struct GetApiQuotaRequest {
    cgi_path: String,
}

#[derive(Debug, Clone, Serialize)]
struct ClearApiQuotaRequest {
    cgi_path: String,
}

#[derive(Debug, Clone, Serialize)]
struct ClearQuotaByAppSecretRequest {
    appid: String,
    appsecret: String,
}

#[derive(Debug, Clone, Serialize)]
struct GetRidInfoRequest {
    rid: String,
}

#[derive(Debug, Clone, Serialize)]
struct CallbackCheckRequest {
    action: String,
    check_operator: String,
}

// ============================================================================
// Internal Response Types
// ============================================================================

/// Internal response for endpoints that only return errcode/errmsg
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

/// API quota details
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct QuotaInfo {
    /// Daily API call limit
    #[serde(default)]
    pub daily_limit: i64,
    /// Number of calls used today
    #[serde(default)]
    pub used: i64,
    /// Remaining calls today
    #[serde(default)]
    pub remain: i64,
}

/// Response from getApiQuota
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiQuotaResponse {
    /// Quota details
    #[serde(default)]
    pub quota: QuotaInfo,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Request debug information
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RidRequestInfo {
    /// Request invocation timestamp (Unix epoch seconds)
    #[serde(default)]
    pub invoke_time: i64,
    /// Request cost in milliseconds
    #[serde(default)]
    pub cost_in_ms: i64,
    /// Request URL
    #[serde(default)]
    pub request_url: String,
    /// Request body
    #[serde(default)]
    pub request_body: String,
    /// Response body
    #[serde(default)]
    pub response_body: String,
    /// Client IP address
    #[serde(default)]
    pub client_ip: String,
}

/// Response from getRidInfo
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RidInfoResponse {
    /// Request debug information
    #[serde(default)]
    pub request: RidRequestInfo,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// DNS check result entry
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DnsInfo {
    /// IP address
    #[serde(default)]
    pub ip: String,
    /// Real network operator
    #[serde(default)]
    pub real_operator: String,
}

/// Ping check result entry
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PingInfo {
    /// IP address
    #[serde(default)]
    pub ip: String,
    /// Source operator
    #[serde(default)]
    pub from_operator: String,
    /// Packet loss rate
    #[serde(default)]
    pub package_loss: String,
    /// Response time
    #[serde(default)]
    pub time: String,
}

/// Response from callbackCheck
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CallbackCheckResponse {
    /// DNS check results
    #[serde(default)]
    pub dns: Vec<DnsInfo>,
    /// Ping check results
    #[serde(default)]
    pub ping: Vec<PingInfo>,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Response from getApiDomainIp and getCallbackIp
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IpListResponse {
    /// List of IP addresses
    #[serde(default)]
    pub ip_list: Vec<String>,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

// ============================================================================
// OpenApiApi
// ============================================================================

/// OpenAPI Management API
///
/// Provides methods for managing API quotas, debugging, and server info.
pub struct OpenApiApi {
    context: Arc<WechatContext>,
}

impl OpenApiApi {
    /// Create a new OpenApiApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Clear all API call quotas for this appid
    ///
    /// POST /cgi-bin/clear_quota?access_token=ACCESS_TOKEN
    ///
    /// # Returns
    /// `Ok(())` on success
    pub async fn clear_quota(&self) -> Result<(), WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/cgi-bin/clear_quota?access_token={}", access_token);
        let body = ClearQuotaRequest {
            appid: self.context.client.appid().to_string(),
        };
        let response: BaseApiResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(())
    }

    /// Get API call quota for a specific endpoint
    ///
    /// POST /cgi-bin/openapi/quota/get?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `cgi_path` - The API path to query (e.g., "/cgi-bin/message/custom/send")
    pub async fn get_api_quota(&self, cgi_path: &str) -> Result<ApiQuotaResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/cgi-bin/openapi/quota/get?access_token={}", access_token);
        let body = GetApiQuotaRequest {
            cgi_path: cgi_path.to_string(),
        };
        let response: ApiQuotaResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Clear API call quota for a specific endpoint
    ///
    /// POST /cgi-bin/openapi/quota/clear?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `cgi_path` - The API path to clear quota for
    pub async fn clear_api_quota(&self, cgi_path: &str) -> Result<(), WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/cgi-bin/openapi/quota/clear?access_token={}", access_token);
        let body = ClearApiQuotaRequest {
            cgi_path: cgi_path.to_string(),
        };
        let response: BaseApiResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(())
    }

    /// Clear all API call quotas using AppSecret (no access_token required)
    ///
    /// POST /cgi-bin/clear_quota/v2
    ///
    /// This endpoint authenticates with appid + appsecret directly,
    /// bypassing the access_token mechanism.
    pub async fn clear_quota_by_app_secret(&self) -> Result<(), WechatError> {
        let path = "/cgi-bin/clear_quota/v2";
        let body = ClearQuotaByAppSecretRequest {
            appid: self.context.client.appid().to_string(),
            appsecret: self.context.client.secret().to_string(),
        };
        let response: BaseApiResponse = self.context.client.post(path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(())
    }

    /// Get request debug information by rid
    ///
    /// POST /cgi-bin/openapi/rid/get?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `rid` - The request ID to look up
    pub async fn get_rid_info(&self, rid: &str) -> Result<RidInfoResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/cgi-bin/openapi/rid/get?access_token={}", access_token);
        let body = GetRidInfoRequest {
            rid: rid.to_string(),
        };
        let response: RidInfoResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Check callback URL connectivity
    ///
    /// POST /cgi-bin/callback/check?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `action` - Check action type (e.g., "all", "dns", "ping")
    /// * `check_operator` - Operator to check (e.g., "DEFAULT", "CHINANET", "UNICOM", "CAP")
    pub async fn callback_check(
        &self,
        action: &str,
        check_operator: &str,
    ) -> Result<CallbackCheckResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/cgi-bin/callback/check?access_token={}", access_token);
        let body = CallbackCheckRequest {
            action: action.to_string(),
            check_operator: check_operator.to_string(),
        };
        let response: CallbackCheckResponse = self.context.client.post(&path, &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Get WeChat API server IP addresses
    ///
    /// GET /cgi-bin/get_api_domain_ip?access_token=ACCESS_TOKEN
    pub async fn get_api_domain_ip(&self) -> Result<IpListResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = "/cgi-bin/get_api_domain_ip";
        let query = [("access_token", access_token.as_str())];
        let response: IpListResponse = self.context.client.get(path, &query).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Get WeChat callback server IP addresses
    ///
    /// GET /cgi-bin/getcallbackip?access_token=ACCESS_TOKEN
    pub async fn get_callback_ip(&self) -> Result<IpListResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = "/cgi-bin/getcallbackip";
        let query = [("access_token", access_token.as_str())];
        let response: IpListResponse = self.context.client.get(path, &query).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for OpenApiApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "openapi"
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

    // ---- Deserialization Tests ----

    #[test]
    fn test_api_quota_response_parse() {
        let json = r#"{
            "quota": {
                "daily_limit": 10000000,
                "used": 500,
                "remain": 9999500
            },
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: ApiQuotaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.quota.daily_limit, 10_000_000);
        assert_eq!(response.quota.used, 500);
        assert_eq!(response.quota.remain, 9_999_500);
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_api_quota_response_missing_quota() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: ApiQuotaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.quota.daily_limit, 0);
        assert_eq!(response.quota.used, 0);
        assert_eq!(response.quota.remain, 0);
    }

    #[test]
    fn test_rid_info_response_parse() {
        let json = r#"{
            "request": {
                "invoke_time": 1635927298,
                "cost_in_ms": 100,
                "request_url": "access_token=xxx",
                "request_body": "{\"appid\":\"wx1234\"}",
                "response_body": "{\"errcode\":0}",
                "client_ip": "1.2.3.4"
            },
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: RidInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.request.invoke_time, 1_635_927_298);
        assert_eq!(response.request.cost_in_ms, 100);
        assert_eq!(response.request.client_ip, "1.2.3.4");
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_rid_info_response_missing_request() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: RidInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.request.invoke_time, 0);
        assert!(response.request.client_ip.is_empty());
    }

    #[test]
    fn test_callback_check_response_parse() {
        let json = r#"{
            "dns": [
                {"ip": "1.2.3.4", "real_operator": "unicom"}
            ],
            "ping": [
                {"ip": "1.2.3.4", "from_operator": "cap", "package_loss": "0%", "time": "20.536ms"}
            ],
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: CallbackCheckResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.dns.len(), 1);
        assert_eq!(response.dns[0].ip, "1.2.3.4");
        assert_eq!(response.dns[0].real_operator, "unicom");
        assert_eq!(response.ping.len(), 1);
        assert_eq!(response.ping[0].from_operator, "cap");
        assert_eq!(response.ping[0].package_loss, "0%");
        assert_eq!(response.ping[0].time, "20.536ms");
    }

    #[test]
    fn test_callback_check_response_empty_arrays() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: CallbackCheckResponse = serde_json::from_str(json).unwrap();
        assert!(response.dns.is_empty());
        assert!(response.ping.is_empty());
    }

    #[test]
    fn test_ip_list_response_parse() {
        let json = r#"{
            "ip_list": ["101.226.62.77", "101.226.62.78", "101.226.62.79"],
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: IpListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.ip_list.len(), 3);
        assert_eq!(response.ip_list[0], "101.226.62.77");
    }

    #[test]
    fn test_ip_list_response_empty() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: IpListResponse = serde_json::from_str(json).unwrap();
        assert!(response.ip_list.is_empty());
    }

    #[test]
    fn test_api_name() {
        let context = create_test_context("http://localhost:0");
        let api = OpenApiApi::new(context);
        assert_eq!(api.api_name(), "openapi");
    }

    // ---- Wiremock Integration Tests ----

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

    #[tokio::test]
    async fn test_clear_quota_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/clear_quota"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = OpenApiApi::new(context);
        let result = api.clear_quota().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_quota_api_error() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/clear_quota"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 40013,
                "errmsg": "invalid appid"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = OpenApiApi::new(context);
        let result = api.clear_quota().await;
        assert!(result.is_err());
        if let Err(WechatError::Api { code, message }) = result {
            assert_eq!(code, 40013);
            assert_eq!(message, "invalid appid");
        } else {
            panic!("Expected WechatError::Api");
        }
    }

    #[tokio::test]
    async fn test_get_api_quota_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/openapi/quota/get"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "quota": {
                    "daily_limit": 10000000,
                    "used": 500,
                    "remain": 9999500
                },
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = OpenApiApi::new(context);
        let result = api.get_api_quota("/cgi-bin/message/custom/send").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.quota.daily_limit, 10_000_000);
        assert_eq!(response.quota.used, 500);
    }

    #[tokio::test]
    async fn test_clear_quota_by_app_secret_success() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        // No token mock needed — this endpoint doesn't use access_token

        Mock::given(method("POST"))
            .and(path("/cgi-bin/clear_quota/v2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = OpenApiApi::new(context);
        let result = api.clear_quota_by_app_secret().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_rid_info_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/openapi/rid/get"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "request": {
                    "invoke_time": 1635927298,
                    "cost_in_ms": 100,
                    "request_url": "/cgi-bin/clear_quota",
                    "request_body": "",
                    "response_body": "{\"errcode\":0}",
                    "client_ip": "1.2.3.4"
                },
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = OpenApiApi::new(context);
        let result = api
            .get_rid_info("61234567-abcd-1234-abcd-123456789012")
            .await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.request.invoke_time, 1_635_927_298);
        assert_eq!(response.request.cost_in_ms, 100);
        assert_eq!(response.request.client_ip, "1.2.3.4");
    }

    #[tokio::test]
    async fn test_callback_check_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/callback/check"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "dns": [{"ip": "1.2.3.4", "real_operator": "unicom"}],
                "ping": [{"ip": "1.2.3.4", "from_operator": "cap", "package_loss": "0%", "time": "20.536ms"}],
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = OpenApiApi::new(context);
        let result = api.callback_check("all", "DEFAULT").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.dns.len(), 1);
        assert_eq!(response.ping.len(), 1);
    }

    #[tokio::test]
    async fn test_get_api_domain_ip_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("GET"))
            .and(path("/cgi-bin/get_api_domain_ip"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ip_list": ["101.226.62.77", "101.226.62.78"],
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = OpenApiApi::new(context);
        let result = api.get_api_domain_ip().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.ip_list.len(), 2);
        assert_eq!(response.ip_list[0], "101.226.62.77");
    }

    #[tokio::test]
    async fn test_get_callback_ip_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        setup_token_mock(&mock_server).await;

        Mock::given(method("GET"))
            .and(path("/cgi-bin/getcallbackip"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ip_list": ["101.226.103.61", "101.226.103.62"],
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = OpenApiApi::new(context);
        let result = api.get_callback_ip().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.ip_list.len(), 2);
        assert_eq!(response.ip_list[0], "101.226.103.61");
    }
}
