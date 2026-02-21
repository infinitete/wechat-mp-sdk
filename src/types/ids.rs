use serde::{Deserialize, Serialize};

/// WeChat Mini Program AppID (18 characters)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppId(String);

impl AppId {
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        if !id.starts_with("wx") {
            return Err(format!("AppId must start with 'wx', got {}", id));
        }
        if id.len() != 18 {
            return Err(format!("AppId must be 18 characters, got {}", id.len()));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// WeChat Mini Program AppSecret
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppSecret(String);

impl AppSecret {
    pub fn new(secret: impl Into<String>) -> Result<Self, String> {
        let secret = secret.into();
        if secret.is_empty() {
            return Err("AppSecret must not be empty".to_string());
        }
        Ok(Self(secret))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// WeChat Mini Program OpenID (20-40 characters)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpenId(String);

impl OpenId {
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        if id.is_empty() || id.len() < 20 || id.len() > 40 {
            return Err(format!("OpenId must be 20-40 characters, got {}", id.len()));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// WeChat UnionID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnionId(String);

impl UnionId {
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        if id.is_empty() {
            return Err("UnionId must not be empty".to_string());
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// WeChat Session Key (base64 encoded, typically 24 characters)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionKey(String);

impl SessionKey {
    pub fn new(key: impl Into<String>) -> Result<Self, String> {
        let key = key.into();
        if key.is_empty() {
            return Err("SessionKey must not be empty".to_string());
        }
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// WeChat Access Token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn new(token: impl Into<String>) -> Result<Self, String> {
        let token = token.into();
        if token.is_empty() {
            return Err("AccessToken must not be empty".to_string());
        }
        Ok(Self(token))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
        assert!(result.unwrap_err().contains("must start with 'wx'"));
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
        let key = "abcdefghijklmnopqrstuvwx".to_string();
        let session_key = SessionKey::new(key.clone()).unwrap();
        assert_eq!(session_key.as_str(), key);
    }

    #[test]
    fn test_session_key_empty() {
        let result = SessionKey::new("");
        assert!(result.is_err());
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
}
