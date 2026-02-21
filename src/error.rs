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

use thiserror::Error;

/// WeChat SDK error types
///
/// # Variants
///
/// - `Http`: HTTP request errors
/// - `Json`: JSON serialization/deserialization errors
/// - `Api`: WeChat API returned an error
/// - `Token`: Access token related errors
/// - `Config`: Configuration errors
/// - `Signature`: Signature verification errors
/// - `Crypto`: Cryptography operation errors
/// - `NotSupported`: Feature not supported
#[derive(Debug, Error)]
pub enum WechatError {
    /// HTTP request error
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

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

    /// Feature not supported
    ///
    /// Some features may not be available in the current SDK version
    /// or are not supported by the WeChat API.
    #[error("Feature not supported: {0}")]
    NotSupported(String),
}
