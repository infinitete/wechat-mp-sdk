//! Access token management for WeChat API
//!
//! Handles token caching, automatic refresh, and concurrency safety.

use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::Mutex;

use crate::client::WechatClient;
use crate::error::WechatError;
use crate::types::AccessToken;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

struct CachedToken {
    token: AccessToken,
    expires_at: Instant,
}

impl CachedToken {
    fn is_expired(&self, buffer: Duration) -> bool {
        Instant::now() + buffer >= self.expires_at
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

impl TokenResponse {
    fn is_success(&self) -> bool {
        self.errcode == 0
    }
}

/// Manages access_token lifecycle with automatic refresh
pub struct TokenManager {
    client: WechatClient,
    cache: Mutex<Option<CachedToken>>,
    refresh_buffer: Duration,
}

impl TokenManager {
    pub fn new(client: WechatClient) -> Self {
        Self {
            client,
            cache: Mutex::new(None),
            refresh_buffer: Duration::from_secs(5 * 60),
        }
    }

    pub async fn get_token(&self) -> Result<String, WechatError> {
        let mut cache = self.cache.lock().await;

        if let Some(ref cached) = *cache {
            if !cached.is_expired(self.refresh_buffer) {
                return Ok(cached.token.as_str().to_string());
            }
        }

        let response = self.fetch_token_with_retry().await?;

        let token = AccessToken::new(response.access_token).map_err(WechatError::Token)?;

        let cached = CachedToken {
            token: token.clone(),
            expires_at: Instant::now() + Duration::from_secs(response.expires_in),
        };

        *cache = Some(cached);
        Ok(token.as_str().to_string())
    }

    async fn fetch_token_with_retry(&self) -> Result<TokenResponse, WechatError> {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match self.fetch_token().await {
                Ok(response) => {
                    if response.is_success() {
                        return Ok(response);
                    }
                    return Err(WechatError::Api {
                        code: response.errcode,
                        message: response.errmsg,
                    });
                }
                Err(WechatError::Http(e)) => {
                    last_error = Some(WechatError::Http(e));
                    if attempt < MAX_RETRIES - 1 {
                        tokio::time::sleep(Duration::from_millis(
                            RETRY_DELAY_MS * (attempt + 1) as u64,
                        ))
                        .await;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or_else(|| WechatError::Token("Unknown error".to_string())))
    }

    async fn fetch_token(&self) -> Result<TokenResponse, WechatError> {
        let path = "/cgi-bin/token";
        let query = [
            ("grant_type", "client_credential"),
            ("appid", self.client.appid()),
            ("secret", self.client.secret()),
        ];

        self.client.get(path, &query).await
    }

    pub async fn invalidate(&self) {
        let mut cache = self.cache.lock().await;
        *cache = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AppId, AppSecret};

    fn create_test_client() -> WechatClient {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .build()
            .unwrap()
    }

    #[test]
    fn test_token_manager_creation() {
        let client = create_test_client();
        let manager = TokenManager::new(client);
        assert!(manager.cache.try_lock().unwrap().is_none());
    }

    #[test]
    fn test_cached_token_not_expired() {
        let token = AccessToken::new("test_token").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(7200),
        };
        assert!(!cached.is_expired(Duration::from_secs(300)));
    }

    #[test]
    fn test_cached_token_expired() {
        let token = AccessToken::new("test_token").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(100),
        };
        assert!(cached.is_expired(Duration::from_secs(300)));
    }

    #[test]
    fn test_token_response_success() {
        let response = TokenResponse {
            access_token: "token123".to_string(),
            expires_in: 7200,
            errcode: 0,
            errmsg: String::new(),
        };
        assert!(response.is_success());
    }

    #[test]
    fn test_token_response_error() {
        let response = TokenResponse {
            access_token: String::new(),
            expires_in: 0,
            errcode: 40001,
            errmsg: "invalid credential".to_string(),
        };
        assert!(!response.is_success());
    }

    #[tokio::test]
    async fn test_invalidate() {
        let client = create_test_client();
        let manager = TokenManager::new(client);

        let token = AccessToken::new("test").unwrap();
        let cached = CachedToken {
            token,
            expires_at: Instant::now() + Duration::from_secs(7200),
        };
        *manager.cache.lock().await = Some(cached);

        manager.invalidate().await;

        assert!(manager.cache.lock().await.is_none());
    }
}
