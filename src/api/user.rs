use serde::{Deserialize, Serialize};

use crate::client::WechatClient;
use crate::error::WechatError;
use crate::token::TokenManager;

pub use crate::types::Watermark;

/// User information from WeChat
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    /// User's nickname
    #[serde(default)]
    pub nick_name: Option<String>,
    /// Avatar URL
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// Gender: 0=unknown, 1=male, 2=female
    #[serde(default)]
    pub gender: u8,
    /// City
    #[serde(default)]
    pub city: Option<String>,
    /// Province
    #[serde(default)]
    pub province: Option<String>,
    /// Country
    #[serde(default)]
    pub country: Option<String>,
    /// Language
    #[serde(default)]
    pub language: Option<String>,
}

/// Phone number information from getPhoneNumber API
#[derive(Debug, Clone, Deserialize)]
pub struct PhoneInfo {
    /// User's phone number (with country code, e.g., +8613800138000)
    pub phone_number: String,
    /// Pure phone number without country code (e.g., 13800138000)
    pub pure_phone_number: String,
    /// Country code (e.g., 86)
    pub country_code: String,
    /// Watermark information
    pub watermark: Watermark,
}

/// Response from getPhoneNumber API
#[derive(Debug, Clone, Deserialize)]
pub struct PhoneNumberResponse {
    /// Phone info
    pub phone_info: PhoneInfo,
    /// Error code
    #[serde(default)]
    pub errcode: i32,
    /// Error message
    #[serde(default)]
    pub errmsg: String,
}

/// Request for getPhoneNumber API
#[derive(Debug, Serialize)]
struct PhoneNumberRequest {
    code: String,
}

/// User API for retrieving user information
pub struct UserApi {
    client: WechatClient,
}

impl UserApi {
    /// Create a new UserApi instance
    pub fn new(client: WechatClient) -> Self {
        Self { client }
    }

    /// Get user's phone number
    ///
    /// Calls the getPhoneNumber API with the code obtained from the client.
    /// Requires a valid access_token.
    ///
    /// # Arguments
    /// * `token_manager` - Token manager to get access_token
    /// * `code` - The code obtained from button open-type="getPhoneNumber"
    ///
    /// # Returns
    /// PhoneNumberResponse containing phone_info with phone number details
    pub async fn get_phone_number(
        &self,
        token_manager: &TokenManager,
        code: &str,
    ) -> Result<PhoneNumberResponse, WechatError> {
        let access_token = token_manager.get_token().await?;

        let path = format!(
            "/wxa/business/getuserphonenumber?access_token={}",
            access_token
        );
        let request = PhoneNumberRequest {
            code: code.to_string(),
        };

        let response: PhoneNumberResponse = self.client.post(&path, &request).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(response)
    }
}
