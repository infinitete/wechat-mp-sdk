use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::WechatError;

fn contains_control_chars(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_control())
}

fn is_whitespace_only(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_whitespace())
}

fn has_leading_trailing_whitespace(s: &str) -> bool {
    s != s.trim()
}

fn validate_base64_and_decode(s: &str) -> Result<Vec<u8>, String> {
    let valid_chars = |c: char| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=';
    if !s.chars().all(valid_chars) {
        return Err("contains invalid base64 characters".to_string());
    }
    BASE64_STANDARD
        .decode(s)
        .map_err(|e| format!("invalid base64: {}", e))
}

/// WeChat Mini Program AppID (18 characters)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppId(String);

impl AppId {
    pub fn new(id: impl Into<String>) -> Result<Self, WechatError> {
        let id = id.into();
        if !id.starts_with("wx") {
            return Err(WechatError::InvalidAppId(format!(
                "AppId must start with 'wx', got {}",
                id
            )));
        }
        if id.len() != 18 {
            return Err(WechatError::InvalidAppId(format!(
                "AppId must be 18 characters, got {}",
                id.len()
            )));
        }
        Ok(Self(id))
    }

    /// Creates an AppId without validation.
    ///
    /// This is a safe function — no undefined behavior occurs with invalid input,
    /// but subsequent API calls may fail if the value is not a valid WeChat AppId.
    /// Prefer [`AppId::new`] for user-supplied input.
    #[must_use]
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// WeChat Mini Program AppSecret
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppSecret(String);

impl fmt::Debug for AppSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AppSecret(****)")
    }
}

impl AppSecret {
    pub fn new(secret: impl Into<String>) -> Result<Self, WechatError> {
        let secret = secret.into();
        if secret.is_empty() {
            return Err(WechatError::InvalidAppSecret(
                "AppSecret must not be empty".to_string(),
            ));
        }
        if is_whitespace_only(&secret) {
            return Err(WechatError::InvalidAppSecret(
                "AppSecret must not be whitespace-only".to_string(),
            ));
        }
        if contains_control_chars(&secret) {
            return Err(WechatError::InvalidAppSecret(
                "AppSecret must not contain control characters".to_string(),
            ));
        }
        Ok(Self(secret))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AppSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

/// WeChat Mini Program OpenID (20-40 characters)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpenId(String);

impl OpenId {
    pub fn new(id: impl Into<String>) -> Result<Self, WechatError> {
        let id = id.into();
        if id.is_empty() || id.len() < 20 || id.len() > 40 {
            return Err(WechatError::InvalidOpenId(format!(
                "OpenId must be 20-40 characters, got {}",
                id.len()
            )));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OpenId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// WeChat UnionID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnionId(String);

impl UnionId {
    pub fn new(id: impl Into<String>) -> Result<Self, WechatError> {
        let id = id.into();
        if id.is_empty() {
            return Err(WechatError::InvalidUnionId(
                "UnionId must not be empty".to_string(),
            ));
        }
        if is_whitespace_only(&id) {
            return Err(WechatError::InvalidUnionId(
                "UnionId must not be whitespace-only".to_string(),
            ));
        }
        if contains_control_chars(&id) {
            return Err(WechatError::InvalidUnionId(
                "UnionId must not contain control characters".to_string(),
            ));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UnionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// WeChat Session Key (base64 encoded, typically 24 characters)
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionKey(String);

impl fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionKey(****)")
    }
}

impl SessionKey {
    pub fn new(key: impl Into<String>) -> Result<Self, WechatError> {
        let key = key.into();
        if key.is_empty() {
            return Err(WechatError::InvalidSessionKey(
                "SessionKey must not be empty".to_string(),
            ));
        }
        if is_whitespace_only(&key) {
            return Err(WechatError::InvalidSessionKey(
                "SessionKey must not be whitespace-only".to_string(),
            ));
        }
        if has_leading_trailing_whitespace(&key) {
            return Err(WechatError::InvalidSessionKey(
                "SessionKey must not have leading/trailing whitespace".to_string(),
            ));
        }
        if contains_control_chars(&key) {
            return Err(WechatError::InvalidSessionKey(
                "SessionKey must not contain control characters".to_string(),
            ));
        }
        let decoded = validate_base64_and_decode(&key)
            .map_err(|e| WechatError::InvalidSessionKey(format!("SessionKey {}", e)))?;
        if decoded.len() != 16 {
            return Err(WechatError::InvalidSessionKey(format!(
                "SessionKey must decode to 16 bytes for AES-128, got {}",
                decoded.len()
            )));
        }
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

/// WeChat Access Token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn new(token: impl Into<String>) -> Result<Self, WechatError> {
        let token = token.into();
        if token.is_empty() {
            return Err(WechatError::InvalidAccessToken(
                "AccessToken must not be empty".to_string(),
            ));
        }
        if is_whitespace_only(&token) {
            return Err(WechatError::InvalidAccessToken(
                "AccessToken must not be whitespace-only".to_string(),
            ));
        }
        if contains_control_chars(&token) {
            return Err(WechatError::InvalidAccessToken(
                "AccessToken must not contain control characters".to_string(),
            ));
        }
        if has_leading_trailing_whitespace(&token) {
            return Err(WechatError::InvalidAccessToken(
                "AccessToken must not have leading/trailing whitespace".to_string(),
            ));
        }
        Ok(Self(token))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_id_valid() {
        let id = "wx1234567890abcdef".to_string();
        let app_id = AppId::new(id.clone()).unwrap();
        assert_eq!(app_id.as_str(), id);
    }

    #[test]
    fn test_app_id_invalid_length() {
        let result = AppId::new("short");
        assert!(result.is_err());
    }

    #[test]
    fn test_app_id_invalid_prefix() {
        let result = AppId::new("abcdefghijklmnop");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("must start with 'wx'"));
    }

    #[test]
    fn test_app_secret_valid() {
        let secret = "abc123".to_string();
        let app_secret = AppSecret::new(secret.clone()).unwrap();
        assert_eq!(app_secret.as_str(), secret);
    }

    #[test]
    fn test_app_secret_empty() {
        let result = AppSecret::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_app_secret_debug_redacted() {
        let secret = AppSecret::new("super_secret_value").unwrap();
        let debug_output = format!("{:?}", secret);
        assert_eq!(debug_output, "AppSecret(****)");
        assert!(!debug_output.contains("super_secret_value"));
    }

    #[test]
    fn test_open_id_valid() {
        let id20 = "o1234567890123456789".to_string();
        assert_eq!(id20.len(), 20);
        assert!(OpenId::new(id20).is_ok());

        let id40 = "o123456789012345678901234567890123456789".to_string();
        assert_eq!(id40.len(), 40);
        assert!(OpenId::new(id40).is_ok());

        let id28 = "o123456789012345678901234567".to_string();
        assert_eq!(id28.len(), 28);
        assert!(OpenId::new(id28).is_ok());
    }

    #[test]
    fn test_open_id_invalid_length() {
        assert!(OpenId::new("").is_err());

        let short = "o123456789012345678".to_string();
        assert_eq!(short.len(), 19);
        assert!(OpenId::new(short).is_err());

        let long = "o1234567890123456789012345678901234567890".to_string();
        assert_eq!(long.len(), 41);
        assert!(OpenId::new(long).is_err());
    }

    #[test]
    fn test_union_id_valid() {
        let id = "union1234567890".to_string();
        let union_id = UnionId::new(id.clone()).unwrap();
        assert_eq!(union_id.as_str(), id);
    }

    #[test]
    fn test_union_id_empty() {
        let result = UnionId::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_key_valid() {
        // Valid base64 that decodes to 16 bytes for AES-128
        // "YWJjZGVmZ2hpamtsbW5vcA==" = base64("abcdefghijklmnop")
        let key = "YWJjZGVmZ2hpamtsbW5vcA==".to_string();
        let session_key = SessionKey::new(key.clone()).unwrap();
        assert_eq!(session_key.as_str(), key);
    }

    #[test]
    fn test_session_key_empty() {
        let result = SessionKey::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_key_debug_redacted() {
        let key = SessionKey::new("YWJjZGVmZ2hpamtsbW5vcA==").unwrap();
        let debug_output = format!("{:?}", key);
        assert_eq!(debug_output, "SessionKey(****)");
        assert!(!debug_output.contains("YWJjZGVmZ2hpamtsbW5vcA=="));
    }

    #[test]
    fn test_access_token_valid() {
        let token = "token1234567890abcdef".to_string();
        let access_token = AccessToken::new(token.clone()).unwrap();
        assert_eq!(access_token.as_str(), token);
    }

    #[test]
    fn test_access_token_empty() {
        let result = AccessToken::new("");
        assert!(result.is_err());
    }

    // ========================================================================
    // BOUNDARY TESTS - ID/Token Validation (hardened)
    // ========================================================================

    // -----------------------------------------------------------------
    // SessionKey boundary tests
    // -----------------------------------------------------------------

    #[test]
    fn test_session_key_whitespace_only() {
        let result = SessionKey::new("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_key_with_whitespace_prefix_suffix() {
        let result = SessionKey::new("  YWJjZGVmZ2hpamtsbW5vcA==  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_key_control_characters() {
        let result = SessionKey::new("abc\x00\x01def");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_key_invalid_base64() {
        let result = SessionKey::new("invalid!!base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_key_valid_base64_wrong_length() {
        // "YWJj" decodes to "abc" (3 bytes) - invalid for AES-128
        let result = SessionKey::new("YWJj");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------
    // AppSecret boundary tests
    // -----------------------------------------------------------------

    #[test]
    fn test_app_secret_whitespace_only() {
        let result = AppSecret::new("   \t\n   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_app_secret_control_characters() {
        let result = AppSecret::new("secret\x00\x01\x02");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------
    // UnionId boundary tests
    // -----------------------------------------------------------------

    #[test]
    fn test_union_id_whitespace_only() {
        let result = UnionId::new("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_union_id_control_characters() {
        let result = UnionId::new("union\x00\x01id");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------
    // AccessToken boundary tests
    // -----------------------------------------------------------------

    #[test]
    fn test_access_token_whitespace_only() {
        let result = AccessToken::new("   \t\n   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_access_token_control_characters() {
        let result = AccessToken::new("token\x00\x01value");
        assert!(result.is_err());
    }

    #[test]
    fn test_access_token_with_leading_trailing_whitespace() {
        let result = AccessToken::new("  token_value_123  ");
        assert!(result.is_err());
    }

    // ========================================================================
    // COMPATIBILITY MATRIX - Validation now hardened
    // ========================================================================
    //
    // | Type         | Valid Input                    | Accepts |
    // |--------------|--------------------------------|---------|
    // | SessionKey   | Valid base64, 16 bytes decoded| YES     |
    // | SessionKey   | Empty/whitespace/invalid      | NO      |
    // | AppSecret    | Non-empty, no control chars   | YES     |
    // | AppSecret    | Empty/whitespace-only/ctrl    | NO      |
    // | UnionId      | Non-empty, no control chars   | YES     |
    // | UnionId      | Empty/whitespace-only/ctrl    | NO      |
    // | AccessToken  | Non-empty, no ws/ctrl         | YES     |
    // | AccessToken  | Empty/whitespace/ctrl/ws-ws   | NO      |
    // ========================================================================

    // ========================================================================
    // DISPLAY TRAIT TESTS
    // ========================================================================

    #[test]
    fn test_display_app_id() {
        let id = AppId::new("wx1234567890abcdef").unwrap();
        assert_eq!(format!("{}", id), "wx1234567890abcdef");
    }

    #[test]
    fn test_display_open_id() {
        let id = OpenId::new("o1234567890123456789").unwrap();
        assert_eq!(format!("{}", id), "o1234567890123456789");
    }

    #[test]
    fn test_display_app_secret_redacted() {
        let secret = AppSecret::new("my_secret_value").unwrap();
        assert_eq!(format!("{}", secret), "***");
    }

    #[test]
    fn test_display_session_key_redacted() {
        let key = SessionKey::new("YWJjZGVmZ2hpamtsbW5vcA==").unwrap();
        assert_eq!(format!("{}", key), "***");
    }

    #[test]
    fn test_display_union_id() {
        let id = UnionId::new("union1234567890").unwrap();
        assert_eq!(format!("{}", id), "union1234567890");
    }

    #[test]
    fn test_display_access_token() {
        let token = AccessToken::new("token1234567890abcdef").unwrap();
        assert_eq!(format!("{}", token), "token1234567890abcdef");
    }
}
