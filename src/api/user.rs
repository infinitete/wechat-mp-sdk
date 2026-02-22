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

#[derive(Debug, Serialize)]
struct PhoneNumberRequest {
    code: String,
}

#[derive(Debug, Serialize)]
struct PluginOpenPIdRequest {
    code: String,
}

#[derive(Debug, Serialize)]
struct CheckEncryptedDataRequest {
    encrypted_msg_hash: String,
}

#[derive(Debug, Serialize)]
struct UserEncryptKeyRequest {
    openid: String,
    signature: String,
    sig_method: String,
}

/// Response from getPluginOpenPId
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginOpenPIdResponse {
    #[serde(default)]
    pub openpid: String,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Response from checkEncryptedData
///
/// Note: WeChat API returns "vaild" (typo), not "valid"
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckEncryptedDataResponse {
    /// Whether the encrypted data is valid (WeChat uses "vaild" in their API)
    #[serde(default)]
    pub vaild: bool,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Response from getPaidUnionid
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaidUnionIdResponse {
    #[serde(default)]
    pub unionid: String,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Encrypt key info entry
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EncryptKeyInfo {
    #[serde(default)]
    pub encrypt_key: String,
    #[serde(default)]
    pub version: i32,
    #[serde(default)]
    pub expire_in: i64,
    #[serde(default)]
    pub iv: String,
    #[serde(default)]
    pub create_time: i64,
}

/// Response from getUserEncryptKey
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserEncryptKeyResponse {
    #[serde(default)]
    pub key_info_list: Vec<EncryptKeyInfo>,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

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
        let request = PhoneNumberRequest {
            code: code.to_string(),
        };

        let response: PhoneNumberResponse = self
            .context
            .authed_post("/wxa/business/getuserphonenumber", &request)
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response)
    }

    /// Get plugin user's OpenPId
    ///
    /// POST /wxa/getpluginopenpid?access_token=ACCESS_TOKEN
    pub async fn get_plugin_open_pid(
        &self,
        code: &str,
    ) -> Result<PluginOpenPIdResponse, WechatError> {
        let body = PluginOpenPIdRequest {
            code: code.to_string(),
        };
        let response: PluginOpenPIdResponse = self
            .context
            .authed_post("/wxa/getpluginopenpid", &body)
            .await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Check if encrypted data has been tampered with
    ///
    /// POST /wxa/business/checkencryptedmsg?access_token=ACCESS_TOKEN
    pub async fn check_encrypted_data(
        &self,
        encrypted_msg_hash: &str,
    ) -> Result<CheckEncryptedDataResponse, WechatError> {
        let body = CheckEncryptedDataRequest {
            encrypted_msg_hash: encrypted_msg_hash.to_string(),
        };
        let response: CheckEncryptedDataResponse = self
            .context
            .authed_post("/wxa/business/checkencryptedmsg", &body)
            .await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Get unionid for a user who has made a payment
    ///
    /// GET /wxa/getpaidunionid?access_token=ACCESS_TOKEN
    pub async fn get_paid_unionid(
        &self,
        openid: &str,
        transaction_id: &str,
    ) -> Result<PaidUnionIdResponse, WechatError> {
        let response: PaidUnionIdResponse = self
            .context
            .authed_get(
                "/wxa/getpaidunionid",
                &[("openid", openid), ("transaction_id", transaction_id)],
            )
            .await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Get user's encrypt key for encrypted data
    ///
    /// POST /wxa/business/getuserencryptkey?access_token=ACCESS_TOKEN
    pub async fn get_user_encrypt_key(
        &self,
        openid: &str,
        signature: &str,
        sig_method: &str,
    ) -> Result<UserEncryptKeyResponse, WechatError> {
        let body = UserEncryptKeyRequest {
            openid: openid.to_string(),
            signature: signature.to_string(),
            sig_method: sig_method.to_string(),
        };
        let response: UserEncryptKeyResponse = self
            .context
            .authed_post("/wxa/business/getuserencryptkey", &body)
            .await?;
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

    #[test]
    fn test_plugin_open_pid_response_parse() {
        let json = r#"{"openpid": "openpid_abc123", "errcode": 0, "errmsg": "ok"}"#;
        let response: PluginOpenPIdResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.openpid, "openpid_abc123");
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_check_encrypted_data_response_parse() {
        let json = r#"{"vaild": true, "errcode": 0, "errmsg": "ok"}"#;
        let response: CheckEncryptedDataResponse = serde_json::from_str(json).unwrap();
        assert!(response.vaild);
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_paid_unionid_response_parse() {
        let json = r#"{"unionid": "union_abc123", "errcode": 0, "errmsg": "ok"}"#;
        let response: PaidUnionIdResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.unionid, "union_abc123");
    }

    #[test]
    fn test_user_encrypt_key_response_parse() {
        let json = r#"{
            "key_info_list": [
                {
                    "encrypt_key": "key123",
                    "version": 1,
                    "expire_in": 3600,
                    "iv": "iv123",
                    "create_time": 1700000000
                }
            ],
            "errcode": 0,
            "errmsg": "ok"
        }"#;
        let response: UserEncryptKeyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.key_info_list.len(), 1);
        assert_eq!(response.key_info_list[0].encrypt_key, "key123");
        assert_eq!(response.key_info_list[0].version, 1);
        assert_eq!(response.key_info_list[0].expire_in, 3600);
    }

    #[test]
    fn test_user_encrypt_key_response_defaults() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: UserEncryptKeyResponse = serde_json::from_str(json).unwrap();
        assert!(response.key_info_list.is_empty());
    }
}
