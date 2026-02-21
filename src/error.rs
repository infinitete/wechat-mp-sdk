//! WeChat SDK error types
//!
//! This module defines error types for the WeChat Mini Program SDK.
//!
//! ## Common WeChat API Error Codes
//!
//! - `-1`: System error
//! - `0`: Success
//! - `-1000`: Sign error
//! - `-1001`: Invalid parameter
//! - `-1002`: AppID error
//! - `-1003`: Access token error
//! - `-1004`: API frequency limit exceeded
//! - `-1005`: Permission denied
//! - `-1006`: API call failed
//! - `40001`: Invalid credential (access_token)
//! - `40002`: Invalid grant_type
//! - `40013`: Invalid appid
//! - `40125`: Invalid appsecret

use std::fmt;
use std::sync::Arc;
use thiserror::Error;

/// HTTP/transport error wrapper
///
/// Wraps either a reqwest HTTP error or a response decode error.
#[derive(Debug)]
pub enum HttpError {
    /// Reqwest HTTP client error
    Reqwest(Arc<reqwest::Error>),
    /// Response body decode error (valid JSON but doesn't match expected type)
    Decode(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::Reqwest(e) => write!(f, "{}", e),
            HttpError::Decode(msg) => write!(f, "Response decode error: {}", msg),
        }
    }
}

impl Clone for HttpError {
    fn clone(&self) -> Self {
        match self {
            HttpError::Reqwest(e) => HttpError::Reqwest(Arc::clone(e)),
            HttpError::Decode(msg) => HttpError::Decode(msg.clone()),
        }
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HttpError::Reqwest(e) => Some(e.as_ref()),
            HttpError::Decode(_) => None,
        }
    }
}

impl From<reqwest::Error> for HttpError {
    fn from(e: reqwest::Error) -> Self {
        HttpError::Reqwest(Arc::new(e))
    }
}

/// WeChat SDK error types
///
/// # Variants
///
/// - `Http`: HTTP request/response errors
/// - `Json`: JSON serialization/deserialization errors
/// - `Api`: WeChat API returned an error
/// - `Token`: Access token related errors
/// - `Config`: Configuration errors
/// - `Signature`: Signature verification errors
/// - `Crypto`: Cryptography operation errors
/// - `InvalidAppId`: Invalid AppId format
/// - `InvalidOpenId`: Invalid OpenId format
/// - `InvalidAccessToken`: Invalid access token
/// - `InvalidAppSecret`: Invalid AppSecret
/// - `InvalidSessionKey`: Invalid SessionKey
/// - `InvalidUnionId`: Invalid UnionId
#[derive(Debug, Error)]
pub enum WechatError {
    /// HTTP request/response error (includes decode errors)
    #[error("{0}")]
    Http(HttpError),

    /// JSON serialization/deserialization error
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// WeChat API returned an error
    ///
    /// # Fields
    /// - `code`: Error code returned by WeChat API
    /// - `message`: Error message from WeChat API
    #[error("WeChat API error (code={code}): {message}")]
    Api { code: i32, message: String },

    /// Access token related error
    #[error("Access token error: {0}")]
    Token(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    Signature(String),

    /// Cryptography operation error
    #[error("Crypto operation error: {0}")]
    Crypto(String),

    /// Invalid AppId format
    ///
    /// AppId must start with 'wx' and be 18 characters long
    #[error("Invalid AppId: {0}")]
    InvalidAppId(String),

    /// Invalid OpenId format
    ///
    /// OpenId must be 20-40 characters
    #[error("Invalid OpenId: {0}")]
    InvalidOpenId(String),

    /// Invalid AccessToken
    #[error("Invalid AccessToken: {0}")]
    InvalidAccessToken(String),

    /// Invalid AppSecret
    #[error("Invalid AppSecret: {0}")]
    InvalidAppSecret(String),

    /// Invalid SessionKey
    #[error("Invalid SessionKey: {0}")]
    InvalidSessionKey(String),

    /// Invalid UnionId
    #[error("Invalid UnionId: {0}")]
    InvalidUnionId(String),
}

impl Clone for WechatError {
    fn clone(&self) -> Self {
        match self {
            WechatError::Http(e) => WechatError::Http(e.clone()),
            WechatError::Json(e) => WechatError::Json(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))),
            WechatError::Api { code, message } => WechatError::Api {
                code: *code,
                message: message.clone(),
            },
            WechatError::Token(msg) => WechatError::Token(msg.clone()),
            WechatError::Config(msg) => WechatError::Config(msg.clone()),
            WechatError::Signature(msg) => WechatError::Signature(msg.clone()),
            WechatError::Crypto(msg) => WechatError::Crypto(msg.clone()),
            WechatError::InvalidAppId(msg) => WechatError::InvalidAppId(msg.clone()),
            WechatError::InvalidOpenId(msg) => WechatError::InvalidOpenId(msg.clone()),
            WechatError::InvalidAccessToken(msg) => WechatError::InvalidAccessToken(msg.clone()),
            WechatError::InvalidAppSecret(msg) => WechatError::InvalidAppSecret(msg.clone()),
            WechatError::InvalidSessionKey(msg) => WechatError::InvalidSessionKey(msg.clone()),
            WechatError::InvalidUnionId(msg) => WechatError::InvalidUnionId(msg.clone()),
        }
    }
}

impl WechatError {
    /// Check WeChat API response errcode, return error if non-zero.
    pub(crate) fn check_api(errcode: i32, errmsg: &str) -> Result<(), WechatError> {
        if errcode != 0 {
            Err(WechatError::Api {
                code: errcode,
                message: errmsg.to_string(),
            })
        } else {
            Ok(())
        }
    }
}

impl From<reqwest::Error> for WechatError {
    fn from(e: reqwest::Error) -> Self {
        WechatError::Http(HttpError::Reqwest(Arc::new(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_appid_error_message() {
        let err = WechatError::InvalidAppId("invalid".to_string());
        assert_eq!(err.to_string(), "Invalid AppId: invalid");
    }

    #[test]
    fn test_invalid_openid_error_message() {
        let err = WechatError::InvalidOpenId("short".to_string());
        assert_eq!(err.to_string(), "Invalid OpenId: short");
    }

    #[test]
    fn test_invalid_access_token_error_message() {
        let err = WechatError::InvalidAccessToken("".to_string());
        assert_eq!(err.to_string(), "Invalid AccessToken: ");
    }

    #[test]
    fn test_invalid_app_secret_error_message() {
        let err = WechatError::InvalidAppSecret("wrong".to_string());
        assert_eq!(err.to_string(), "Invalid AppSecret: wrong");
    }

    #[test]
    fn test_invalid_session_key_error_message() {
        let err = WechatError::InvalidSessionKey("invalid".to_string());
        assert_eq!(err.to_string(), "Invalid SessionKey: invalid");
    }

    #[test]
    fn test_invalid_union_id_error_message() {
        let err = WechatError::InvalidUnionId("".to_string());
        assert_eq!(err.to_string(), "Invalid UnionId: ");
    }

    #[test]
    fn test_check_api_success() {
        let result = WechatError::check_api(0, "success");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_api_error() {
        let result = WechatError::check_api(40013, "invalid appid");
        assert!(result.is_err());
        if let Err(WechatError::Api { code, message }) = result {
            assert_eq!(code, 40013);
            assert_eq!(message, "invalid appid");
        } else {
            panic!("Expected Api error");
        }
    }

    #[test]
    fn test_wechat_error_clone() {
        let err = WechatError::Api {
            code: 40013,
            message: "invalid appid".to_string(),
        };
        let cloned = err.clone();
        assert_eq!(format!("{}", err), format!("{}", cloned));

        let token_err = WechatError::Token("expired".to_string());
        let cloned_token = token_err.clone();
        assert_eq!(format!("{}", token_err), format!("{}", cloned_token));
    }

    #[test]
    fn test_http_error_clone() {
        let err = HttpError::Decode("bad json".to_string());
        let cloned = err.clone();
        assert_eq!(format!("{}", err), format!("{}", cloned));
    }

    #[test]
    fn test_http_error_source_chain() {
        use std::error::Error;

        let decode_err = HttpError::Decode("test".to_string());
        assert!(decode_err.source().is_none());
    }
}
