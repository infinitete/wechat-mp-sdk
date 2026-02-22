use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::api::r#trait::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize)]
pub struct QrcodeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_color: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_color: Option<LineColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hyaline: Option<bool>,
}

impl QrcodeOptions {
    pub fn new() -> Self {
        Self {
            path: None,
            width: None,
            auto_color: None,
            line_color: None,
            is_hyaline: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct UnlimitQrcodeOptions {
    pub scene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_color: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_color: Option<LineColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hyaline: Option<bool>,
}

impl UnlimitQrcodeOptions {
    pub fn new(scene: impl Into<String>) -> Self {
        Self {
            scene: scene.into(),
            page: None,
            width: None,
            auto_color: None,
            line_color: None,
            is_hyaline: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UrlSchemeExpire {
    #[serde(rename = "type")]
    pub expire_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_interval: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrlSchemeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire: Option<UrlSchemeExpire>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct UrlSchemeResponse {
    pub openlink: String,
    #[serde(default)]
    pub errcode: i32,
    #[serde(default)]
    pub errmsg: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrlLinkOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_interval: Option<i64>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct UrlLinkResponse {
    pub link: String,
    #[serde(default)]
    pub errcode: i32,
    #[serde(default)]
    pub errmsg: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShortLinkOptions {
    pub page_url: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct ShortLinkResponse {
    pub link: String,
    #[serde(default)]
    pub errcode: i32,
    #[serde(default)]
    pub errmsg: String,
}

/// WeChat Mini Program QR code and URL link API
///
/// Provides methods for generating Mini Program codes, QR codes,
/// URL schemes, URL links, and short links.
pub struct QrcodeApi {
    context: Arc<WechatContext>,
}

impl QrcodeApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Generate a Mini Program code (limited usage, up to 100,000 codes).
    ///
    /// POST /wxa/getwxacode
    pub async fn get_wxa_code(&self, options: QrcodeOptions) -> Result<Vec<u8>, WechatError> {
        self.get_image_bytes("/wxa/getwxacode", &options).await
    }

    /// Generate an unlimited Mini Program code (no usage limit).
    ///
    /// POST /wxa/getwxacodeunlimit
    pub async fn get_wxa_code_unlimit(
        &self,
        options: UnlimitQrcodeOptions,
    ) -> Result<Vec<u8>, WechatError> {
        self.get_image_bytes("/wxa/getwxacodeunlimit", &options)
            .await
    }

    /// Create a Mini Program QR code for a given page path.
    ///
    /// POST /cgi-bin/wxaapp/createwxaqrcode
    pub async fn create_qrcode(
        &self,
        path: &str,
        width: Option<u32>,
    ) -> Result<Vec<u8>, WechatError> {
        #[derive(Serialize)]
        struct Request {
            path: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            width: Option<u32>,
        }

        let request = Request {
            path: path.to_string(),
            width,
        };
        self.get_image_bytes("/cgi-bin/wxaapp/createwxaqrcode", &request)
            .await
    }

    /// Generate a URL Scheme for opening the Mini Program.
    ///
    /// POST /wxa/generatescheme
    pub async fn generate_url_scheme(
        &self,
        options: UrlSchemeOptions,
    ) -> Result<String, WechatError> {
        let response: UrlSchemeResponse = self
            .context
            .authed_post("/wxa/generatescheme", &options)
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response.openlink)
    }

    /// Generate a URL Link for opening the Mini Program.
    ///
    /// POST /wxa/generate_urllink
    pub async fn generate_url_link(&self, options: UrlLinkOptions) -> Result<String, WechatError> {
        let response: UrlLinkResponse = self
            .context
            .authed_post("/wxa/generate_urllink", &options)
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response.link)
    }

    /// Generate a short link for the Mini Program.
    ///
    /// POST /wxa/genwxashortlink
    pub async fn generate_short_link(
        &self,
        options: ShortLinkOptions,
    ) -> Result<String, WechatError> {
        let response: ShortLinkResponse = self
            .context
            .authed_post("/wxa/genwxashortlink", &options)
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response.link)
    }

    /// Query details of an existing URL Scheme
    ///
    /// POST /wxa/queryscheme?access_token=ACCESS_TOKEN
    pub async fn query_scheme(&self, scheme: &str) -> Result<QuerySchemeResponse, WechatError> {
        #[derive(Serialize)]
        struct Request {
            scheme: String,
        }

        let body = Request {
            scheme: scheme.to_string(),
        };
        let response: QuerySchemeResponse =
            self.context.authed_post("/wxa/queryscheme", &body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Query details of an existing URL Link
    ///
    /// POST /wxa/query_urllink?access_token=ACCESS_TOKEN
    pub async fn query_url_link(
        &self,
        url_link: &str,
    ) -> Result<QueryUrlLinkResponse, WechatError> {
        #[derive(Serialize)]
        struct Request {
            url_link: String,
        }

        let body = Request {
            url_link: url_link.to_string(),
        };
        let response: QueryUrlLinkResponse = self
            .context
            .authed_post("/wxa/query_urllink", &body)
            .await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    /// Generate an NFC Scheme for opening the Mini Program via NFC
    ///
    /// POST /wxa/generatenfcscheme?access_token=ACCESS_TOKEN
    pub async fn generate_nfc_scheme(
        &self,
        options: NfcSchemeOptions,
    ) -> Result<NfcSchemeResponse, WechatError> {
        let response: NfcSchemeResponse = self
            .context
            .authed_post("/wxa/generatenfcscheme", &options)
            .await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    async fn get_image_bytes<T: Serialize>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<Vec<u8>, WechatError> {
        let response = self.context.authed_post_raw(endpoint, body).await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if content_type.contains("application/json") {
            let error: ErrorResponse = response.json().await?;
            return Err(WechatError::Api {
                code: error.errcode,
                message: error.errmsg,
            });
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl WechatApi for QrcodeApi {
    fn api_name(&self) -> &'static str {
        "qrcode"
    }

    fn context(&self) -> &WechatContext {
        &self.context
    }
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    errcode: i32,
    errmsg: String,
}

/// Scheme info from queryScheme
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SchemeInfo {
    #[serde(default)]
    pub appid: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub create_time: i64,
    #[serde(default)]
    pub expire_time: i64,
    #[serde(default)]
    pub env_version: String,
}

/// Scheme quota info
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SchemeQuota {
    #[serde(default)]
    pub long_time_used: i64,
    #[serde(default)]
    pub long_time_limit: i64,
}

/// Response from queryScheme
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuerySchemeResponse {
    #[serde(default)]
    pub scheme_info: SchemeInfo,
    #[serde(default)]
    pub scheme_quota: SchemeQuota,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// URL Link info from queryUrlLink
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UrlLinkInfo {
    #[serde(default)]
    pub appid: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub create_time: i64,
    #[serde(default)]
    pub expire_time: i64,
    #[serde(default)]
    pub env_version: String,
}

/// URL Link quota info
#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UrlLinkQuota {
    #[serde(default)]
    pub long_time_used: i64,
    #[serde(default)]
    pub long_time_limit: i64,
}

/// Response from queryUrlLink
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryUrlLinkResponse {
    #[serde(default)]
    pub url_link_info: UrlLinkInfo,
    #[serde(default)]
    pub url_link_quota: UrlLinkQuota,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

/// Jump target for NFC Scheme
#[derive(Debug, Clone, Serialize)]
pub struct NfcSchemeJumpWxa {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_version: Option<String>,
}

/// Options for generating NFC Scheme
#[derive(Debug, Clone, Serialize)]
pub struct NfcSchemeOptions {
    pub jump_wxa: NfcSchemeJumpWxa,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sn: Option<String>,
}

/// Response from generateNFCScheme
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NfcSchemeResponse {
    #[serde(default)]
    pub openlink: String,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qrcode_options_defaults() {
        let mut options = QrcodeOptions::new();
        options.path = Some("/pages/index".to_string());
        assert!(options.path.is_some());
    }

    #[test]
    fn test_line_color() {
        let color = LineColor { r: 0, g: 0, b: 0 };
        assert_eq!(color.r, 0);
    }

    #[test]
    fn test_unlimit_options() {
        let options = UnlimitQrcodeOptions {
            scene: "abc".to_string(),
            page: Some("/pages/index".to_string()),
            width: Some(430),
            auto_color: None,
            line_color: None,
            is_hyaline: None,
        };
        assert_eq!(options.scene, "abc");
    }

    #[test]
    fn test_query_scheme_response_parse() {
        let json = r#"{
            "scheme_info": {
                "appid": "wx1234567890abcdef",
                "path": "/pages/index",
                "query": "id=123",
                "create_time": 1700000000,
                "expire_time": 1700100000,
                "env_version": "release"
            },
            "scheme_quota": {
                "long_time_used": 5,
                "long_time_limit": 100
            },
            "errcode": 0,
            "errmsg": "ok"
        }"#;
        let response: QuerySchemeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.scheme_info.appid, "wx1234567890abcdef");
        assert_eq!(response.scheme_info.path, "/pages/index");
        assert_eq!(response.scheme_quota.long_time_used, 5);
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_query_url_link_response_parse() {
        let json = r#"{
            "url_link_info": {
                "appid": "wx1234567890abcdef",
                "path": "/pages/index",
                "query": "",
                "create_time": 1700000000,
                "expire_time": 1700100000,
                "env_version": "release"
            },
            "url_link_quota": {
                "long_time_used": 2,
                "long_time_limit": 100
            },
            "errcode": 0,
            "errmsg": "ok"
        }"#;
        let response: QueryUrlLinkResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.url_link_info.appid, "wx1234567890abcdef");
        assert_eq!(response.url_link_quota.long_time_used, 2);
    }

    #[test]
    fn test_nfc_scheme_response_parse() {
        let json =
            r#"{"openlink": "weixin://dl/business/?t=NFC123", "errcode": 0, "errmsg": "ok"}"#;
        let response: NfcSchemeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.openlink, "weixin://dl/business/?t=NFC123");
        assert_eq!(response.errcode, 0);
    }

    #[test]
    fn test_query_scheme_response_defaults() {
        let json = r#"{"errcode": 0, "errmsg": "ok"}"#;
        let response: QuerySchemeResponse = serde_json::from_str(json).unwrap();
        assert!(response.scheme_info.appid.is_empty());
        assert_eq!(response.scheme_quota.long_time_used, 0);
    }
}
