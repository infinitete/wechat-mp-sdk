use serde::{Deserialize, Serialize};

use crate::error::WechatError;

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

    /// Verify watermark timestamp is within allowed skew from current time.
    pub fn verify_timestamp_freshness(
        &self,
        now_timestamp: i64,
        max_skew_seconds: i64,
    ) -> Result<(), WechatError> {
        if max_skew_seconds < 0 {
            return Err(WechatError::Signature(
                "Watermark max_skew must be non-negative".to_string(),
            ));
        }

        let skew_seconds = self.timestamp.abs_diff(now_timestamp);
        let max_skew_seconds = max_skew_seconds as u64;

        if skew_seconds > max_skew_seconds {
            return Err(WechatError::Signature(format!(
                "Watermark timestamp stale: skew {}s exceeds max_skew {}s",
                skew_seconds, max_skew_seconds
            )));
        }

        Ok(())
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

    #[test]
    fn test_verify_timestamp_freshness_within_skew() {
        let wm = Watermark::new(1_700_000_000, "wx1234567890abcdef");
        let result = wm.verify_timestamp_freshness(1_700_000_120, 180);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_timestamp_freshness_rejects_stale_timestamp() {
        let wm = Watermark::new(1_700_000_000, "wx1234567890abcdef");
        let result = wm.verify_timestamp_freshness(1_700_000_301, 300);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_timestamp_freshness_rejects_negative_max_skew() {
        let wm = Watermark::new(1_700_000_000, "wx1234567890abcdef");
        let result = wm.verify_timestamp_freshness(1_700_000_000, -1);
        assert!(result.is_err());
    }
}
