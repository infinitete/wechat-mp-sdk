use serde::{Deserialize, Serialize};

/// WeChat Mini Program AppID (18 characters)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppId(String);

impl AppId {
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
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

/// WeChat Mini Program OpenID (28 characters)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpenId(String);

impl OpenId {
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        if id.len() != 28 {
            return Err(format!("OpenId must be 28 characters, got {}", id.len()));
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
        let id = "oXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string(); // 28 chars: 1 + 27 Xs
        assert_eq!(id.len(), 28);
        let open_id = OpenId::new(id.clone()).unwrap();
        assert_eq!(open_id.as_str(), id);
    }

    #[test]
    fn test_open_id_invalid_length() {
        let result = OpenId::new("short");
        assert!(result.is_err());
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
