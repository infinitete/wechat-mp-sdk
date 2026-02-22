//! Cloud Base API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct InvokeCloudFunctionRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct DelayedFunctionTaskRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct CloudDatabaseRequest {
    pub query: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct UploadFileLinkRequest {
    pub env: String,
    pub path: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct DownloadFileLinkRequest {
    pub env: String,
    pub file_list: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct DeleteCloudFileRequest {
    pub env: String,
    pub fileid_list: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct SendCloudBaseSmsRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloudResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct CloudApi {
    context: Arc<WechatContext>,
}

impl CloudApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn invoke_cloud_function(
        &self,
        request: &InvokeCloudFunctionRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/invokecloudfunction", request).await
    }

    pub async fn add_delayed_function_task(
        &self,
        request: &DelayedFunctionTaskRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/adddelayedfunctiontask", request).await
    }

    pub async fn database_add(
        &self,
        request: &CloudDatabaseRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/databaseadd", request).await
    }

    pub async fn database_delete(
        &self,
        request: &CloudDatabaseRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/databasedelete", request).await
    }

    pub async fn database_update(
        &self,
        request: &CloudDatabaseRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/databaseupdate", request).await
    }

    pub async fn database_query(
        &self,
        request: &CloudDatabaseRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/databasequery", request).await
    }

    pub async fn get_upload_file_link(
        &self,
        request: &UploadFileLinkRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/uploadfile", request).await
    }

    pub async fn get_download_file_link(
        &self,
        request: &DownloadFileLinkRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/batchdownloadfile", request).await
    }

    pub async fn delete_cloud_file(
        &self,
        request: &DeleteCloudFileRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/batchdeletefile", request).await
    }

    pub async fn new_send_cloud_base_sms(
        &self,
        request: &SendCloudBaseSmsRequest,
    ) -> Result<CloudResponse, WechatError> {
        self.post_json("/tcb/sendsms_v2", request).await
    }

    async fn post_json<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<CloudResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(endpoint, &access_token);
        let response: CloudResponse = self.context.client.post(&path, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for CloudApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "cloud"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","request_id":"x"}"#;
        let response: CloudResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert!(response.extra.contains_key("request_id"));
    }
}
