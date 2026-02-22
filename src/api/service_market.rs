//! Service Market API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct InvokeServiceRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceMarketResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct ServiceMarketApi {
    context: Arc<WechatContext>,
}

impl ServiceMarketApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn invoke_service(
        &self,
        request: &InvokeServiceRequest,
    ) -> Result<ServiceMarketResponse, WechatError> {
        let response: ServiceMarketResponse = self
            .context
            .authed_post("/wxa/servicemarket", request)
            .await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for ServiceMarketApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "service_market"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_market_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","service":"x"}"#;
        let response: ServiceMarketResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
    }
}
