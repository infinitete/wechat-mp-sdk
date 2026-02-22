//! Live Streaming API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct LiveRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct DeleteRoomRequest {
    pub id: i32,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct GetLiveInfoRequest {
    pub start: i32,
    pub limit: i32,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiveResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct LiveApi {
    context: Arc<WechatContext>,
}

impl LiveApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn create_room(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxaapi/broadcast/room/create", request)
            .await
    }

    pub async fn delete_room(
        &self,
        request: &DeleteRoomRequest,
    ) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxaapi/broadcast/room/deleteroom", request)
            .await
    }

    pub async fn edit_room(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxaapi/broadcast/room/editroom", request)
            .await
    }

    pub async fn get_live_info(
        &self,
        request: &GetLiveInfoRequest,
    ) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxa/business/getliveinfo", request).await
    }

    pub async fn add_goods(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxaapi/broadcast/goods/add", request).await
    }

    pub async fn update_goods_info(
        &self,
        request: &LiveRequest,
    ) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxaapi/broadcast/goods/update", request)
            .await
    }

    pub async fn delete_goods_info(
        &self,
        request: &LiveRequest,
    ) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxaapi/broadcast/goods/delete", request)
            .await
    }

    pub async fn push_message(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxaapi/broadcast/subscribe/send", request)
            .await
    }

    pub async fn get_followers(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        self.post_json("/wxaapi/broadcast/subscribe/get", request)
            .await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<LiveResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(endpoint, &access_token);
        let response: LiveResponse = self.context.client.post(&path, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for LiveApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "live"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","roomid":1}"#;
        let response: LiveResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert!(response.extra.contains_key("roomid"));
    }
}
