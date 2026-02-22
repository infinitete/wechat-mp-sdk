//! Face Verification API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct GetVerifyIdRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct QueryVerifyInfoRequest {
    pub verify_token: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FaceResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct FaceApi {
    context: Arc<WechatContext>,
}

impl FaceApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn get_verify_id(
        &self,
        request: &GetVerifyIdRequest,
    ) -> Result<FaceResponse, WechatError> {
        self.post_json("/cgi-bin/soter/mp/verify_id/get", request)
            .await
    }

    pub async fn query_verify_info(
        &self,
        request: &QueryVerifyInfoRequest,
    ) -> Result<FaceResponse, WechatError> {
        self.post_json("/cgi-bin/soter/mp/verify_result/get", request)
            .await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<FaceResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(endpoint, &access_token);
        let response: FaceResponse = self.context.client.post(&path, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for FaceApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "face"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","verify_result":"pass"}"#;
        let response: FaceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
    }
}
