//! WeChat Search API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct SubmitPagesRequest {
    pub pages: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmitPagesResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct WxsearchApi {
    context: Arc<WechatContext>,
}

impl WxsearchApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn submit_pages(
        &self,
        request: &SubmitPagesRequest,
    ) -> Result<SubmitPagesResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!(
            "/wxa/search/wxaapi_submitpages?access_token={}",
            access_token
        );
        let response: SubmitPagesResponse = self.context.client.post(&path, request).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for WxsearchApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "wxsearch"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_pages_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","ret":0}"#;
        let response: SubmitPagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
    }
}
