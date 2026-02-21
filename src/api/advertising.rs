//! Advertising API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct AdvertisingRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdvertisingResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct AdvertisingApi {
    context: Arc<WechatContext>,
}

impl AdvertisingApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn add_user_action(
        &self,
        request: &AdvertisingRequest,
    ) -> Result<AdvertisingResponse, WechatError> {
        self.post_json("/marketing/add_user_action", request).await
    }

    pub async fn add_user_action_set(
        &self,
        request: &AdvertisingRequest,
    ) -> Result<AdvertisingResponse, WechatError> {
        self.post_json("/marketing/add_user_action_set", request)
            .await
    }

    pub async fn get_user_action_set_reports(
        &self,
        request: &AdvertisingRequest,
    ) -> Result<AdvertisingResponse, WechatError> {
        self.post_json("/marketing/get_user_action_set_reports", request)
            .await
    }

    pub async fn get_user_action_sets(
        &self,
        request: &AdvertisingRequest,
    ) -> Result<AdvertisingResponse, WechatError> {
        self.post_json("/marketing/get_user_action_sets", request)
            .await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<AdvertisingResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("{}?access_token={}", endpoint, access_token);
        let response: AdvertisingResponse = self.context.client.post(&path, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for AdvertisingApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "advertising"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advertising_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","report":[1]}"#;
        let response: AdvertisingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
    }
}
