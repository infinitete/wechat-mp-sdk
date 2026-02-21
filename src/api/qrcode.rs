use serde::{Deserialize, Serialize};

use crate::client::WechatClient;
use crate::error::WechatError;
use crate::token::TokenManager;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct ShortLinkResponse {
    pub link: String,
    #[serde(default)]
    pub errcode: i32,
    #[serde(default)]
    pub errmsg: String,
}

pub struct QrcodeApi {
    client: WechatClient,
}

impl QrcodeApi {
    pub fn new(client: WechatClient) -> Self {
        Self { client }
    }

    pub async fn get_wxa_code(
        &self,
        token_manager: &TokenManager,
        options: QrcodeOptions,
    ) -> Result<Vec<u8>, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxa/getwxacode?access_token={}", access_token);
        self.get_image_bytes(&path, &options).await
    }

    pub async fn get_wxa_code_unlimit(
        &self,
        token_manager: &TokenManager,
        options: UnlimitQrcodeOptions,
    ) -> Result<Vec<u8>, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxa/getwxacodeunlimit?access_token={}", access_token);
        self.get_image_bytes(&path, &options).await
    }

    pub async fn create_qrcode(
        &self,
        token_manager: &TokenManager,
        path: String,
        width: Option<u32>,
    ) -> Result<Vec<u8>, WechatError> {
        let access_token = token_manager.get_token().await?;
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

        let request = Request { path, width };
        self.get_image_bytes(&url, &request).await
    }

    pub async fn generate_url_scheme(
        &self,
        token_manager: &TokenManager,
        options: UrlSchemeOptions,
    ) -> Result<String, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxa/generatescheme?access_token={}", access_token);

        let response: UrlSchemeResponse = self.client.post(&path, &options).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(response.openlink)
    }

    pub async fn generate_url_link(
        &self,
        token_manager: &TokenManager,
        options: UrlLinkOptions,
    ) -> Result<String, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxa/generate_urllink?access_token={}", access_token);

        let response: UrlLinkResponse = self.client.post(&path, &options).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(response.link)
    }

    pub async fn generate_short_link(
        &self,
        token_manager: &TokenManager,
        options: ShortLinkOptions,
    ) -> Result<String, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxa/genwxashortlink?access_token={}", access_token);

        let response: ShortLinkResponse = self.client.post(&path, &options).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(response.link)
    }

    async fn get_image_bytes<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<Vec<u8>, WechatError> {
        let url = format!("{}{}", self.client.base_url(), path);
        let response = self.client.http().post(&url).json(body).send().await?;

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
        let options = QrcodeOptions {
            path: Some("/pages/index".to_string()),
            width: None,
            auto_color: None,
            line_color: None,
            is_hyaline: None,
        };
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
