//! Plugin API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct ManagePluginApplicationRequest {
    pub action: String,
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct ManagePluginRequest {
    pub action: String,
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct PluginApi {
    context: Arc<WechatContext>,
}

impl PluginApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn manage_plugin_application(
        &self,
        request: &ManagePluginApplicationRequest,
    ) -> Result<PluginResponse, WechatError> {
        self.post_plugin(request).await
    }

    pub async fn manage_plugin(
        &self,
        request: &ManagePluginRequest,
    ) -> Result<PluginResponse, WechatError> {
        self.post_plugin(request).await
    }

    async fn post_plugin<B: Serialize>(&self, body: &B) -> Result<PluginResponse, WechatError> {
        let response: PluginResponse = self.context.authed_post("/wxa/plugin", body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for PluginApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "plugin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","data":{}}"#;
        let response: PluginResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
    }
}
