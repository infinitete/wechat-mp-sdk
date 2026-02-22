//! Instant Delivery API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct DeliveryRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeliveryResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct DeliveryApi {
    context: Arc<WechatContext>,
}

impl DeliveryApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn get_all_imme_delivery(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        self.post_json("/cgi-bin/express/local/business/delivery/getall", request)
            .await
    }

    pub async fn pre_add_order(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        self.post_json("/cgi-bin/express/local/business/order/pre_add", request)
            .await
    }

    pub async fn pre_cancel_order(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        self.post_json("/cgi-bin/express/local/business/order/precancel", request)
            .await
    }

    pub async fn add_local_order(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        self.post_json("/cgi-bin/express/local/business/order/add", request)
            .await
    }

    pub async fn cancel_local_order(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        self.post_json("/cgi-bin/express/local/business/order/cancel", request)
            .await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<DeliveryResponse, WechatError> {
        let response: DeliveryResponse = self.context.authed_post(endpoint, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for DeliveryApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "delivery"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","order_id":"x"}"#;
        let response: DeliveryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert!(response.extra.contains_key("order_id"));
    }
}
