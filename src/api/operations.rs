//! Operations API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct EmptyRequest {}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct RealtimeLogSearchRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct FeedbackRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct FeedbackMediaRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct JsErrDetailRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct JsErrListRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OperationsResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct OperationsApi {
    context: Arc<WechatContext>,
}

impl OperationsApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn get_domain_info(&self) -> Result<OperationsResponse, WechatError> {
        self.post_json("/wxa/get_wxa_domain", &EmptyRequest {})
            .await
    }

    pub async fn get_performance(&self) -> Result<OperationsResponse, WechatError> {
        self.post_json("/wxaapi/log/get_performance", &EmptyRequest {})
            .await
    }

    pub async fn get_scene_list(&self) -> Result<OperationsResponse, WechatError> {
        self.get_json("/wxaapi/log/get_scene").await
    }

    pub async fn get_version_list(&self) -> Result<OperationsResponse, WechatError> {
        self.get_json("/wxaapi/log/get_client_version").await
    }

    pub async fn realtime_log_search(
        &self,
        request: &RealtimeLogSearchRequest,
    ) -> Result<OperationsResponse, WechatError> {
        self.post_json("/wxaapi/userlog/userlog_search", request)
            .await
    }

    pub async fn get_feedback(
        &self,
        request: &FeedbackRequest,
    ) -> Result<OperationsResponse, WechatError> {
        self.post_json("/wxaapi/feedback/list", request).await
    }

    pub async fn get_feedback_media(
        &self,
        request: &FeedbackMediaRequest,
    ) -> Result<OperationsResponse, WechatError> {
        self.post_json("/wxaapi/feedback/media/get", request).await
    }

    pub async fn get_js_err_detail(
        &self,
        request: &JsErrDetailRequest,
    ) -> Result<OperationsResponse, WechatError> {
        self.post_json("/wxaapi/log/jserr_detail", request).await
    }

    pub async fn get_js_err_list(
        &self,
        request: &JsErrListRequest,
    ) -> Result<OperationsResponse, WechatError> {
        self.post_json("/wxaapi/log/jserr_list", request).await
    }

    pub async fn get_gray_release_plan(&self) -> Result<OperationsResponse, WechatError> {
        self.get_json("/wxa/getgrayreleaseplan").await
    }

    async fn get_json(&self, endpoint: &str) -> Result<OperationsResponse, WechatError> {
        let response: OperationsResponse = self.context.authed_get(endpoint, &[]).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<OperationsResponse, WechatError> {
        let response: OperationsResponse = self.context.authed_post(endpoint, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for OperationsApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "operations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operations_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","items":[1,2]}"#;
        let response: OperationsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert!(response.extra.contains_key("items"));
    }
}
