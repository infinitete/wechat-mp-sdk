//! Hardware Device API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct HardwareRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HardwareResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct HardwareApi {
    context: Arc<WechatContext>,
}

impl HardwareApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn send_hardware_device_message(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        self.post_json("/cgi-bin/message/device/subscribe/send", request)
            .await
    }

    pub async fn get_sn_ticket(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        self.post_json("/wxa/business/hardware/sn_ticket/get", request)
            .await
    }

    pub async fn create_iot_group_id(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        self.post_json("/wxa/business/hardware/group/create", request)
            .await
    }

    pub async fn get_iot_group_info(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        self.post_json("/wxa/business/hardware/group/get", request)
            .await
    }

    pub async fn add_iot_group_device(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        self.post_json("/wxa/business/hardware/group/device/add", request)
            .await
    }

    pub async fn remove_iot_group_device(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        self.post_json("/wxa/business/hardware/group/device/remove", request)
            .await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<HardwareResponse, WechatError> {
        let response: HardwareResponse = self.context.authed_post(endpoint, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for HardwareApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "hardware"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","ticket":"abc"}"#;
        let response: HardwareResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert!(response.extra.contains_key("ticket"));
    }
}
