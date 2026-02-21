use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::api::r#trait::{WechatApi, WechatContext};
use crate::error::WechatError;
use crate::types::Watermark;

/// User information from WeChat
#[non_exhaustive]
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

impl UserInfo {
    pub fn new(nick_name: Option<String>, gender: u8) -> Self {
        Self {
            nick_name,
            avatar_url: None,
            gender,
            city: None,
            province: None,
            country: None,
            language: None,
        }
    }
}

/// Phone number information from getPhoneNumber API
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
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

impl PhoneInfo {
    pub fn new(
        phone_number: impl Into<String>,
        pure_phone_number: impl Into<String>,
        country_code: impl Into<String>,
        watermark: Watermark,
    ) -> Self {
        Self {
            phone_number: phone_number.into(),
            pure_phone_number: pure_phone_number.into(),
            country_code: country_code.into(),
            watermark,
        }
    }
}

/// Response from getPhoneNumber API
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PhoneNumberResponse {
    /// Phone info
    pub phone_info: PhoneInfo,
    /// Error code
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

impl PhoneNumberResponse {
    /// Creates a new success PhoneNumberResponse. Defaults errcode=0, errmsg="".
    pub fn new(phone_info: PhoneInfo) -> Self {
        Self {
            phone_info,
            errcode: 0,
            errmsg: String::new(),
        }
    }

    pub fn errcode(&self) -> i32 {
        self.errcode
    }

    pub fn errmsg(&self) -> &str {
        &self.errmsg
    }
}

/// Request for getPhoneNumber API
#[derive(Debug, Serialize)]
struct PhoneNumberRequest {
    code: String,
}

/// User API for retrieving user information
pub struct UserApi {
    context: Arc<WechatContext>,
}

impl UserApi {
    /// Create a new UserApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Get user's phone number
    ///
    /// Calls the getPhoneNumber API with the code obtained from the client.
    /// Requires a valid access_token.
    ///
    /// # Arguments
    /// * `code` - The code obtained from button open-type="getPhoneNumber"
    ///
    /// # Returns
    /// PhoneNumberResponse containing phone_info with phone number details
    pub async fn get_phone_number(&self, code: &str) -> Result<PhoneNumberResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;

        let path = format!(
            "/wxa/business/getuserphonenumber?access_token={}",
            access_token
        );
        let request = PhoneNumberRequest {
            code: code.to_string(),
        };

        let response: PhoneNumberResponse = self.context.client.post(&path, &request).await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response)
    }
}

impl WechatApi for UserApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "user"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_number_response_parsing() {
        let json = r#"{
            "phone_info": {
                "phone_number": "+8613800138000",
                "pure_phone_number": "13800138000",
                "country_code": "86",
                "watermark": {
                    "timestamp": 1234567890,
                    "appid": "wx1234567890"
                }
            },
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: PhoneNumberResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert_eq!(response.errmsg, "ok");
        assert_eq!(response.phone_info.phone_number, "+8613800138000");
        assert_eq!(response.phone_info.pure_phone_number, "13800138000");
        assert_eq!(response.phone_info.country_code, "86");
        assert_eq!(response.phone_info.watermark.timestamp(), 1234567890);
    }

    #[test]
    fn test_user_info_parsing() {
        let json = r#"{
            "nick_name": "John",
            "avatar_url": "https://example.com/avatar.jpg",
            "gender": 1,
            "city": "Beijing",
            "province": "Beijing",
            "country": "China",
            "language": "zh_CN"
        }"#;

        let user_info: UserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(user_info.nick_name, Some("John".to_string()));
        assert_eq!(
            user_info.avatar_url,
            Some("https://example.com/avatar.jpg".to_string())
        );
        assert_eq!(user_info.gender, 1);
        assert_eq!(user_info.city, Some("Beijing".to_string()));
        assert_eq!(user_info.province, Some("Beijing".to_string()));
        assert_eq!(user_info.country, Some("China".to_string()));
        assert_eq!(user_info.language, Some("zh_CN".to_string()));
    }

    #[test]
    fn test_user_info_optional_fields() {
        let json = r#"{
            "nick_name": "Jane",
            "gender": 2
        }"#;

        let user_info: UserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(user_info.nick_name, Some("Jane".to_string()));
        assert_eq!(user_info.gender, 2);
        assert_eq!(user_info.avatar_url, None);
        assert_eq!(user_info.city, None);
    }

    #[test]
    fn test_phone_number_response_error_parsing() {
        let json = r#"{
            "phone_info": {
                "phone_number": "+8613800138000",
                "pure_phone_number": "13800138000",
                "country_code": "86",
                "watermark": {
                    "timestamp": 1234567890,
                    "appid": "wx1234567890"
                }
            },
            "errcode": 40001,
            "errmsg": "invalid credential"
        }"#;

        let response: PhoneNumberResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 40001);
        assert_eq!(response.errmsg, "invalid credential");
    }
}
