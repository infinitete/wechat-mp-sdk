//! Common API response primitives
//!
//! Shared types and traits for WeChat Mini Program API request/response patterns.
//!
//! ## Overview
//!
//! Most WeChat APIs return JSON responses with common `errcode` / `errmsg` fields.
//! This module provides:
//!
//! - [`WechatApiResponse`] trait for uniform errcode/errmsg checking
//! - [`ApiResponseBase`] struct for simple error-only responses
//! - [`PaginatedRequest`] and [`PaginatedResponse`] for offset/count pagination
//! - [`DateRangeRequest`] for analytics date range queries
//!
//! ## Usage
//!
//! ```rust
//! use wechat_mp_sdk::api::common::{WechatApiResponse, ApiResponseBase};
//!
//! // Parse a minimal error-only response
//! let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
//! let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
//! assert!(resp.check().is_ok());
//! ```

use serde::{Deserialize, Serialize};

use crate::error::WechatError;

/// Trait for WeChat API responses that carry `errcode` / `errmsg`.
///
/// Implement this on any response struct that includes the standard
/// WeChat error fields. The [`check`](WechatApiResponse::check) method
/// provides a one-step parse-and-validate shortcut.
///
/// # Example
///
/// ```rust
/// use wechat_mp_sdk::api::common::WechatApiResponse;
/// use wechat_mp_sdk::error::WechatError;
///
/// #[derive(serde::Deserialize)]
/// struct MyResponse {
///     #[serde(default)]
///     data: String,
///     #[serde(default)]
///     errcode: i32,
///     #[serde(default)]
///     errmsg: String,
/// }
///
/// impl WechatApiResponse for MyResponse {
///     fn errcode(&self) -> i32 { self.errcode }
///     fn errmsg(&self) -> &str { &self.errmsg }
/// }
///
/// let json = r#"{"errcode": 0, "errmsg": "ok", "data": "hello"}"#;
/// let resp: MyResponse = serde_json::from_str(json).unwrap();
/// assert!(resp.check().is_ok());
/// ```
pub trait WechatApiResponse {
    /// Returns the error code from the API response.
    ///
    /// `0` indicates success; any other value is an error.
    fn errcode(&self) -> i32;

    /// Returns the error message from the API response.
    fn errmsg(&self) -> &str;

    /// Check the response for API errors.
    ///
    /// Returns `Ok(())` when `errcode == 0`, otherwise returns
    /// `WechatError::Api` with the code and message.
    fn check(&self) -> Result<(), WechatError> {
        WechatError::check_api(self.errcode(), self.errmsg())
    }

    /// Returns `true` when the response indicates success (`errcode == 0`).
    fn is_success(&self) -> bool {
        self.errcode() == 0
    }
}

/// Minimal API response carrying only `errcode` and `errmsg`.
///
/// Use this for endpoints that return no data beyond success/failure,
/// or as a lightweight way to inspect the error fields before
/// attempting a full parse.
///
/// # Example
///
/// ```rust
/// use wechat_mp_sdk::api::common::{ApiResponseBase, WechatApiResponse};
///
/// let json = r#"{"errcode": 40013, "errmsg": "invalid appid"}"#;
/// let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
/// assert!(!resp.is_success());
/// assert!(resp.check().is_err());
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiResponseBase {
    /// Error code (`0` = success)
    #[serde(default)]
    pub errcode: i32,
    /// Error message
    #[serde(default)]
    pub errmsg: String,
}

impl ApiResponseBase {
    /// Create a success response.
    pub fn success() -> Self {
        Self {
            errcode: 0,
            errmsg: "ok".to_string(),
        }
    }

    /// Create an error response with the given code and message.
    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            errcode: code,
            errmsg: message.into(),
        }
    }
}

impl WechatApiResponse for ApiResponseBase {
    fn errcode(&self) -> i32 {
        self.errcode
    }

    fn errmsg(&self) -> &str {
        &self.errmsg
    }
}

/// Pagination request parameters for list endpoints.
///
/// Many WeChat APIs accept `offset` and `count` to paginate results.
///
/// # Example
///
/// ```rust
/// use wechat_mp_sdk::api::common::PaginatedRequest;
///
/// let req = PaginatedRequest::new(0, 20);
/// assert_eq!(req.offset, 0);
/// assert_eq!(req.count, 20);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedRequest {
    /// Starting position (0-based offset)
    pub offset: u32,
    /// Number of items to return per page
    pub count: u32,
}

impl PaginatedRequest {
    /// Create a new paginated request.
    ///
    /// # Arguments
    /// * `offset` - Starting position (0-based)
    /// * `count` - Number of items per page
    pub fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }

    /// Create a request for the first page with the given page size.
    pub fn first_page(count: u32) -> Self {
        Self { offset: 0, count }
    }
}

impl Default for PaginatedRequest {
    fn default() -> Self {
        Self {
            offset: 0,
            count: 20,
        }
    }
}

/// Paginated list response from WeChat APIs.
///
/// Wraps a list of items together with total count and the standard
/// `errcode`/`errmsg` error fields.
///
/// # Type Parameters
/// * `T` - The type of items in the list
///
/// # Example
///
/// ```rust
/// use wechat_mp_sdk::api::common::{PaginatedResponse, WechatApiResponse};
///
/// let json = r#"{
///     "total_count": 100,
///     "list": [{"name": "a"}, {"name": "b"}],
///     "errcode": 0,
///     "errmsg": "ok"
/// }"#;
///
/// #[derive(serde::Deserialize, Debug)]
/// struct Item { name: String }
///
/// let resp: PaginatedResponse<Item> = serde_json::from_str(json).unwrap();
/// assert_eq!(resp.total_count, 100);
/// assert_eq!(resp.list.len(), 2);
/// assert!(resp.is_success());
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound(deserialize = "T: serde::de::Deserialize<'de>"))]
pub struct PaginatedResponse<T> {
    /// Total number of items available
    #[serde(default)]
    pub total_count: u32,
    /// Items in the current page
    #[serde(default)]
    pub list: Vec<T>,
    /// Error code (`0` = success)
    #[serde(default)]
    pub(crate) errcode: i32,
    /// Error message
    #[serde(default)]
    pub(crate) errmsg: String,
}

impl<T> WechatApiResponse for PaginatedResponse<T> {
    fn errcode(&self) -> i32 {
        self.errcode
    }

    fn errmsg(&self) -> &str {
        &self.errmsg
    }
}

/// Date range request for analytics and statistics APIs.
///
/// WeChat analytics endpoints typically require `begin_date` and `end_date`
/// in `"yyyyMMdd"` format.
///
/// # Example
///
/// ```rust
/// use wechat_mp_sdk::api::common::DateRangeRequest;
///
/// let req = DateRangeRequest::new("20240101", "20240131");
/// assert_eq!(req.begin_date, "20240101");
/// assert_eq!(req.end_date, "20240131");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeRequest {
    /// Start date (inclusive), format: `"yyyyMMdd"`
    pub begin_date: String,
    /// End date (inclusive), format: `"yyyyMMdd"`
    pub end_date: String,
}

impl DateRangeRequest {
    /// Create a new date range request.
    ///
    /// # Arguments
    /// * `begin_date` - Start date in `"yyyyMMdd"` format
    /// * `end_date` - End date in `"yyyyMMdd"` format
    pub fn new(begin_date: impl Into<String>, end_date: impl Into<String>) -> Self {
        Self {
            begin_date: begin_date.into(),
            end_date: end_date.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_base_success() {
        let resp = ApiResponseBase::success();
        assert_eq!(resp.errcode, 0);
        assert_eq!(resp.errmsg, "ok");
        assert!(resp.is_success());
        assert!(resp.check().is_ok());
    }

    #[test]
    fn test_api_response_base_error() {
        let resp = ApiResponseBase::error(40013, "invalid appid");
        assert_eq!(resp.errcode, 40013);
        assert_eq!(resp.errmsg, "invalid appid");
        assert!(!resp.is_success());
        assert!(resp.check().is_err());
    }

    #[test]
    fn test_api_response_base_deserialize_success() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
        assert!(resp.is_success());
    }

    #[test]
    fn test_api_response_base_deserialize_error() {
        let json = r#"{"errcode": 40013, "errmsg": "invalid appid"}"#;
        let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
        assert!(!resp.is_success());
        let err = resp.check().unwrap_err();
        match err {
            WechatError::Api { code, message } => {
                assert_eq!(code, 40013);
                assert_eq!(message, "invalid appid");
            }
            _ => panic!("Expected WechatError::Api"),
        }
    }

    #[test]
    fn test_api_response_base_defaults_on_missing_fields() {
        let json = r#"{}"#;
        let resp: ApiResponseBase = serde_json::from_str(json).unwrap();
        assert_eq!(resp.errcode, 0);
        assert!(resp.errmsg.is_empty());
        assert!(resp.is_success());
    }

    #[test]
    fn test_api_response_base_serialize_roundtrip() {
        let resp = ApiResponseBase::error(40001, "invalid credential");
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: ApiResponseBase = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.errcode, resp.errcode);
        assert_eq!(parsed.errmsg, resp.errmsg);
    }

    #[test]
    fn test_paginated_request_new() {
        let req = PaginatedRequest::new(10, 20);
        assert_eq!(req.offset, 10);
        assert_eq!(req.count, 20);
    }

    #[test]
    fn test_paginated_request_first_page() {
        let req = PaginatedRequest::first_page(50);
        assert_eq!(req.offset, 0);
        assert_eq!(req.count, 50);
    }

    #[test]
    fn test_paginated_request_default() {
        let req = PaginatedRequest::default();
        assert_eq!(req.offset, 0);
        assert_eq!(req.count, 20);
    }

    #[test]
    fn test_paginated_request_serialize() {
        let req = PaginatedRequest::new(5, 10);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["offset"], 5);
        assert_eq!(json["count"], 10);
    }

    #[test]
    fn test_paginated_response_deserialize() {
        #[derive(Debug, Clone, Deserialize, Serialize)]
        struct Item {
            name: String,
        }

        let json = r#"{
            "total_count": 42,
            "list": [{"name": "alpha"}, {"name": "beta"}],
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let resp: PaginatedResponse<Item> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_count, 42);
        assert_eq!(resp.list.len(), 2);
        assert_eq!(resp.list[0].name, "alpha");
        assert_eq!(resp.list[1].name, "beta");
        assert!(resp.is_success());
        assert!(resp.check().is_ok());
    }

    #[test]
    fn test_paginated_response_empty_list() {
        #[derive(Debug, Clone, Deserialize, Serialize)]
        struct Item {
            name: String,
        }

        let json = r#"{
            "total_count": 0,
            "list": [],
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let resp: PaginatedResponse<Item> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_count, 0);
        assert!(resp.list.is_empty());
        assert!(resp.is_success());
    }

    #[test]
    fn test_paginated_response_error() {
        #[derive(Debug, Clone, Deserialize, Serialize)]
        struct Item {
            name: String,
        }

        let json = r#"{
            "errcode": 40001,
            "errmsg": "invalid credential"
        }"#;

        let resp: PaginatedResponse<Item> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_count, 0);
        assert!(resp.list.is_empty());
        assert!(!resp.is_success());
        assert!(resp.check().is_err());
    }

    #[test]
    fn test_paginated_response_defaults_on_missing() {
        #[derive(Debug, Clone, Deserialize, Serialize)]
        struct Item {
            name: String,
        }

        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let resp: PaginatedResponse<Item> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_count, 0);
        assert!(resp.list.is_empty());
    }

    #[test]
    fn test_date_range_request_new() {
        let req = DateRangeRequest::new("20240101", "20240131");
        assert_eq!(req.begin_date, "20240101");
        assert_eq!(req.end_date, "20240131");
    }

    #[test]
    fn test_date_range_request_serialize() {
        let req = DateRangeRequest::new("20240301", "20240307");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["begin_date"], "20240301");
        assert_eq!(json["end_date"], "20240307");
    }

    #[test]
    fn test_date_range_request_deserialize_roundtrip() {
        let req = DateRangeRequest::new("20240601", "20240630");
        let json = serde_json::to_string(&req).unwrap();
        let parsed: DateRangeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.begin_date, req.begin_date);
        assert_eq!(parsed.end_date, req.end_date);
    }

    #[test]
    fn test_wechat_api_response_trait_check_success() {
        let resp = ApiResponseBase::success();
        assert!(resp.check().is_ok());
    }

    #[test]
    fn test_wechat_api_response_trait_check_error_returns_api_error() {
        let resp = ApiResponseBase::error(-1, "system error");
        let err = resp.check().unwrap_err();
        match err {
            WechatError::Api { code, message } => {
                assert_eq!(code, -1);
                assert_eq!(message, "system error");
            }
            _ => panic!("Expected WechatError::Api"),
        }
    }
}
