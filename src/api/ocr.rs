//! Image and OCR API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct OcrImageRequest {
    pub img_url: String,
}

impl OcrImageRequest {
    pub fn new(img_url: impl Into<String>) -> Self {
        Self {
            img_url: img_url.into(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct IdCardOcrRequest {
    pub img_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OcrResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct OcrApi {
    context: Arc<WechatContext>,
}

impl OcrApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn ai_crop(&self, request: &OcrImageRequest) -> Result<OcrResponse, WechatError> {
        self.post_json("/cv/img/aicrop", request).await
    }

    pub async fn scan_qr_code(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        self.post_json("/cv/img/qrcode", request).await
    }

    pub async fn printed_text_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        self.post_json("/cv/ocr/comm", request).await
    }

    pub async fn vehicle_license_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        self.post_json("/cv/ocr/driving", request).await
    }

    pub async fn bank_card_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        self.post_json("/cv/ocr/bankcard", request).await
    }

    pub async fn business_license_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        self.post_json("/cv/ocr/bizlicense", request).await
    }

    pub async fn driver_license_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        self.post_json("/cv/ocr/drivinglicense", request).await
    }

    pub async fn id_card_ocr(
        &self,
        request: &IdCardOcrRequest,
    ) -> Result<OcrResponse, WechatError> {
        self.post_json("/cv/ocr/idcard", request).await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<OcrResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(endpoint, &access_token);
        let response: OcrResponse = self.context.client.post(&path, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for OcrApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "ocr"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocr_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","items":[{"text":"abc"}]}"#;
        let response: OcrResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert!(response.extra.contains_key("items"));
    }
}
