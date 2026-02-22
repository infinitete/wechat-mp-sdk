//! Nearby Mini Program API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct AddNearbyPoiRequest {
    pub poi_id: String,
    pub related_name: String,
    pub related_credential: String,
    pub related_address: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct DeleteNearbyPoiRequest {
    pub poi_id: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct NearbyPoiListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_rows: Option<i32>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct NearbyShowStatusRequest {
    pub is_open: i32,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NearbyResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct NearbyApi {
    context: Arc<WechatContext>,
}

impl NearbyApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn add_nearby_poi(
        &self,
        request: &AddNearbyPoiRequest,
    ) -> Result<NearbyResponse, WechatError> {
        self.post_json("/wxa/addnearbypoi", request).await
    }

    pub async fn delete_nearby_poi(
        &self,
        request: &DeleteNearbyPoiRequest,
    ) -> Result<NearbyResponse, WechatError> {
        self.post_json("/wxa/delnearbypoi", request).await
    }

    pub async fn get_nearby_poi_list(
        &self,
        request: &NearbyPoiListRequest,
    ) -> Result<NearbyResponse, WechatError> {
        self.post_json("/wxa/getnearbypoilist", request).await
    }

    pub async fn set_show_status(
        &self,
        request: &NearbyShowStatusRequest,
    ) -> Result<NearbyResponse, WechatError> {
        self.post_json("/wxa/setnearbypoishowstatus", request).await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<NearbyResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(endpoint, &access_token);
        let response: NearbyResponse = self.context.client.post(&path, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for NearbyApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "nearby"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearby_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","poi_list":[]}"#;
        let response: NearbyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert!(response.extra.contains_key("poi_list"));
    }
}
