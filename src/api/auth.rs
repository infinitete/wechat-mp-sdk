//! WeChat Authentication API
//!
//! Provides login and authentication related APIs.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::api::r#trait::{WechatApi, WechatContext};
use crate::error::WechatError;

/// Login response from code2Session API
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginResponse {
    /// User's unique ID under the Mini Program
    #[serde(default)]
    pub openid: String,
    /// Session key for decrypting user data
    #[serde(default)]
    pub session_key: String,
    /// User's unique ID across WeChat platform (optional)
    #[serde(default)]
    pub unionid: Option<String>,
    /// Error code (0 means success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

impl LoginResponse {
    /// Create a new LoginResponse with the given fields.
    ///
    /// Sets errcode to 0 and errmsg to empty string (success).
    pub fn new(
        openid: impl Into<String>,
        session_key: impl Into<String>,
        unionid: Option<String>,
    ) -> Self {
        Self {
            openid: openid.into(),
            session_key: session_key.into(),
            unionid,
            errcode: 0,
            errmsg: String::new(),
        }
    }

    /// Check if the response indicates success
    pub fn is_success(&self) -> bool {
        self.errcode == 0
    }

    pub fn errcode(&self) -> i32 {
        self.errcode
    }

    pub fn errmsg(&self) -> &str {
        &self.errmsg
    }
}

/// WeChat authentication API
pub struct AuthApi {
    context: Arc<WechatContext>,
}

impl AuthApi {
    /// Create a new AuthApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Login with code from wx.login()
    ///
    /// Calls the code2Session API to exchange a login code for user info.
    ///
    /// # Arguments
    /// * `js_code` - The code obtained from wx.login() on the client
    ///
    /// # Returns
    /// LoginResponse containing openid, session_key, and optionally unionid
    pub async fn login(&self, js_code: &str) -> Result<LoginResponse, WechatError> {
        let path = "/sns/jscode2session";
        let query = [
            ("appid", self.context.client.appid()),
            ("secret", self.context.client.secret()),
            ("js_code", js_code),
            ("grant_type", "authorization_code"),
        ];

        let response: LoginResponse = self.context.client.get(path, &query).await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response)
    }
}

impl WechatApi for AuthApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "auth"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_response_success_parse() {
        let json = r#"{
            "openid": "oABC123xyz",
            "session_key": "test_session_key_abc",
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: LoginResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.openid, "oABC123xyz");
        assert_eq!(response.session_key, "test_session_key_abc");
        assert!(response.is_success());
        assert!(response.unionid.is_none());
    }

    #[test]
    fn test_login_response_with_unionid() {
        let json = r#"{
            "openid": "oABC123xyz",
            "session_key": "test_session_key_abc",
            "unionid": "uABC123union",
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: LoginResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.openid, "oABC123xyz");
        assert_eq!(response.session_key, "test_session_key_abc");
        assert_eq!(response.unionid, Some("uABC123union".to_string()));
        assert!(response.is_success());
    }

    #[test]
    fn test_login_response_error_parse() {
        let json = r#"{
            "errcode": 40013,
            "errmsg": "invalid appid"
        }"#;

        let response: LoginResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 40013);
        assert_eq!(response.errmsg, "invalid appid");
        assert!(!response.is_success());
        assert!(response.openid.is_empty());
        assert!(response.session_key.is_empty());
    }

    #[test]
    fn test_is_success_true_for_zero() {
        let response = LoginResponse {
            openid: "test".to_string(),
            session_key: "test".to_string(),
            unionid: None,
            errcode: 0,
            errmsg: "ok".to_string(),
        };
        assert!(response.is_success());
    }

    #[test]
    fn test_is_success_false_for_nonzero() {
        let response = LoginResponse {
            openid: "test".to_string(),
            session_key: "test".to_string(),
            unionid: None,
            errcode: -1,
            errmsg: "system error".to_string(),
        };
        assert!(!response.is_success());
    }
}
