use serde::{Deserialize, Serialize};

/// Watermark from WeChat encrypted data
///
/// This struct represents the watermark information included in
/// WeChat's encrypted user data and phone number responses.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Watermark {
    /// Timestamp when data was encrypted
    pub timestamp: i64,
    /// AppID that encrypted the data
    pub appid: String,
}
