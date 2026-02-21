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
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/wxa/getwxacode?access_token={}", access_token);
        self.get_image_bytes(&path, &options).await
    }

    /// Generate an unlimited Mini Program code (no usage limit).
    ///
    /// POST /wxa/getwxacodeunlimit
    pub async fn get_wxa_code_unlimit(
        &self,
        options: UnlimitQrcodeOptions,
    ) -> Result<Vec<u8>, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/wxa/getwxacodeunlimit?access_token={}", access_token);
        self.get_image_bytes(&path, &options).await
    }

    /// Create a Mini Program QR code for a given page path.
    ///
    /// POST /cgi-bin/wxaapp/createwxaqrcode
    pub async fn create_qrcode(
        &self,
        path: &str,
        width: Option<u32>,
    ) -> Result<Vec<u8>, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let url = format!(
            "/cgi-bin/wxaapp/createwxaqrcode?access_token={}",
            access_token
        );

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
        self.get_image_bytes(&url, &request).await
    }

    /// Generate a URL Scheme for opening the Mini Program.
    ///
    /// POST /wxa/generatescheme
    pub async fn generate_url_scheme(
        &self,
        options: UrlSchemeOptions,
    ) -> Result<String, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/wxa/generatescheme?access_token={}", access_token);

        let response: UrlSchemeResponse = self.context.client.post(&path, &options).await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response.openlink)
    }

    /// Generate a URL Link for opening the Mini Program.
    ///
    /// POST /wxa/generate_urllink
    pub async fn generate_url_link(&self, options: UrlLinkOptions) -> Result<String, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/wxa/generate_urllink?access_token={}", access_token);

        let response: UrlLinkResponse = self.context.client.post(&path, &options).await?;

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
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("/wxa/genwxashortlink?access_token={}", access_token);

        let response: ShortLinkResponse = self.context.client.post(&path, &options).await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response.link)
    }

    async fn get_image_bytes<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<Vec<u8>, WechatError> {
        let url = format!("{}{}", self.context.client.base_url(), path);
        let request = self.context.client.http().post(&url).json(body).build()?;
        let response = self.context.client.send_request(request).await?;

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
}
