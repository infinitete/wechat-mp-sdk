use serde::{Deserialize, Serialize};

/// Watermark from WeChat encrypted data
///
/// This struct represents the watermark information included in
/// WeChat's encrypted user data and phone number responses.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Watermark {
    /// Timestamp when data was encrypted
    pub(crate) timestamp: i64,
    /// AppID that encrypted the data
    pub(crate) appid: String,
}

impl Watermark {
    pub fn new(timestamp: i64, appid: impl Into<String>) -> Self {
        Self {
            timestamp,
            appid: appid.into(),
        }
    }

    pub fn appid(&self) -> &str {
        &self.appid
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_new() {
        let wm = Watermark::new(1234567890, "wx1234567890abcdef");
        assert_eq!(wm.appid(), "wx1234567890abcdef");
        assert_eq!(wm.timestamp(), 1234567890);
    }

    #[test]
    fn test_watermark_accessors() {
        let wm = Watermark::new(999, "wxtest");
        assert_eq!(wm.appid(), "wxtest");
        assert_eq!(wm.timestamp(), 999);
    }
}
