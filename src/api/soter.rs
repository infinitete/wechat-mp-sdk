//! SOTER Biometric API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct VerifySignatureRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifySignatureResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct SoterApi {
    context: Arc<WechatContext>,
}

impl SoterApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn verify_signature(
        &self,
        request: &VerifySignatureRequest,
    ) -> Result<VerifySignatureResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!(
            "/cgi-bin/soter/verify_signature?access_token={}",
            access_token
        );
        let response: VerifySignatureResponse = self.context.client.post(&path, request).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for SoterApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "soter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_signature_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","is_ok":1}"#;
        let response: VerifySignatureResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
    }
}
