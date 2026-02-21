//! Logistics / Express API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct LogisticsRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogisticsResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct LogisticsApi {
    context: Arc<WechatContext>,
}

impl LogisticsApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn bind_account(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        self.post_json("/cgi-bin/express/business/account/bind", request)
            .await
    }

    pub async fn get_all_account(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        self.post_json("/cgi-bin/express/business/account/getall", request)
            .await
    }

    pub async fn get_all_delivery(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        self.post_json("/cgi-bin/express/business/delivery/getall", request)
            .await
    }

    pub async fn get_order(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        self.post_json("/cgi-bin/express/business/order/get", request)
            .await
    }

    pub async fn add_order(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        self.post_json("/cgi-bin/express/business/order/add", request)
            .await
    }

    pub async fn get_path(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        self.post_json("/cgi-bin/express/business/path/get", request)
            .await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<LogisticsResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("{}?access_token={}", endpoint, access_token);
        let response: LogisticsResponse = self.context.client.post(&path, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for LogisticsApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "logistics"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logistics_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","waybill_id":"x"}"#;
        let response: LogisticsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
    }
}
