//! WeChat Authentication API
//!
//! Provides login and authentication related APIs.

use serde::Deserialize;

use crate::client::WechatClient;
use crate::error::WechatError;

/// Login response from code2Session API
#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    /// User's unique ID under the Mini Program
    pub openid: String,
    /// Session key for decrypting user data
    pub session_key: String,
    /// User's unique ID across WeChat platform (optional)
    #[serde(default)]
    pub unionid: Option<String>,
    /// Error code (0 means success)
    #[serde(default)]
    pub errcode: i32,
    /// Error message
    #[serde(default)]
    pub errmsg: String,
}

impl LoginResponse {
    /// Check if the response indicates success
    pub fn is_success(&self) -> bool {
        self.errcode == 0
    }
}

/// WeChat authentication API
pub struct AuthApi {
    client: WechatClient,
}

impl AuthApi {
    /// Create a new AuthApi instance
    pub fn new(client: WechatClient) -> Self {
        Self { client }
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
            ("appid", self.client.appid()),
            ("secret", self.client.secret()),
            ("js_code", js_code),
            ("grant_type", "authorization_code"),
        ];

        let response: LoginResponse = self.client.get(path, &query).await?;

        if !response.is_success() {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(response)
    }
}
