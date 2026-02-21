use thiserror::Error;

/// WeChat SDK error types
#[derive(Debug, Error)]
pub enum WechatError {
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WeChat API error (code={code}): {message}")]
    Api { code: i32, message: String },

    #[error("Access token error: {0}")]
    Token(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Signature verification failed: {0}")]
    Signature(String),
}
