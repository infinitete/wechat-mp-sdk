//! Subscribe Message API
//!
//! Provides APIs for sending subscribe messages and managing templates.
//!
//! ## Overview
//!
//! Subscribe messages are special messages that users must opt-in to receive.
//! They are commonly used for notifications like order status, reminders, etc.
//!
//! ## Usage
//!
//! ```ignore
//! use wechat_mp_sdk::api::subscribe::{SubscribeApi, SubscribeMessageOptions, SubscribeMessageData, SubscribeMessageValue};
//!
//! // Create the API instance
//! let subscribe_api = SubscribeApi::new(context);
//!
//! // Send a subscribe message
//! let mut data = SubscribeMessageData::new();
//! data.insert("thing1".to_string(), SubscribeMessageValue::new("Order #123"));
//! data.insert("time2".to_string(), SubscribeMessageValue::new("2024-01-01 12:00"));
//!
//! let options = SubscribeMessageOptions {
//!     touser: "user_openid".to_string(),
//!     template_id: "template_id".to_string(),
//!     data,
//!     page: Some("pages/index/index".to_string()),
//!     miniprogram_state: None,
//!     lang: None,
//! };
//!
//! subscribe_api.send(options).await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{WechatApi, WechatContext};
use crate::error::WechatError;
use crate::types::OpenId;

/// Subscribe message data (key-value pairs)
pub type SubscribeMessageData = HashMap<String, SubscribeMessageValue>;

/// Value for subscribe message field
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeMessageValue {
    pub value: String,
}

impl SubscribeMessageValue {
    /// Create a new subscribe message value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// Request for sending subscribe message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessageRequest {
    #[serde(rename = "touser")]
    touser: OpenId,
    #[serde(rename = "template_id")]
    template_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<String>,
    data: SubscribeMessageData,
    #[serde(skip_serializing_if = "Option::is_none")]
    miniprogram_state: Option<MiniProgramState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lang: Option<Lang>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MiniProgramState {
    Developer,
    Trial,
    Formal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Lang {
    #[serde(rename = "zh_CN")]
    ZhCN,
    #[serde(rename = "en_US")]
    EnUS,
    #[serde(rename = "zh_HK")]
    ZhHK,
    #[serde(rename = "zh_TW")]
    ZhTW,
}

/// Options for sending subscribe messages
#[derive(Debug, Clone)]
pub struct SubscribeMessageOptions {
    /// Recipient's OpenID
    pub touser: OpenId,
    /// Template ID
    pub template_id: String,
    /// Template data
    pub data: SubscribeMessageData,
    /// Page to navigate to (optional)
    pub page: Option<String>,
    /// Mini program state: "developer", "trial", or "formal" (optional)
    pub miniprogram_state: Option<MiniProgramState>,
    /// Language: "zh_CN", "en_US", "zh_HK", "zh_TW" (optional)
    pub lang: Option<Lang>,
}

/// Response from subscribe message API
#[derive(Debug, Clone, Deserialize)]
struct SubscribeMessageResponse {
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

/// Template info
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateInfo {
    /// Private template ID
    #[serde(rename = "priTmplId")]
    pub private_template_id: String,
    /// Template title
    pub title: String,
    /// Template content
    pub content: String,
    /// Example content (optional)
    #[serde(default)]
    pub example: Option<String>,
    /// Template type
    #[serde(rename = "type")]
    pub template_type: i32,
}

/// Response from get template list
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateListResponse {
    pub data: Vec<TemplateInfo>,
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

/// Request for add template
#[derive(Debug, Clone, Serialize)]
struct AddTemplateRequest {
    tid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kid_list: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scene_desc: Option<String>,
}

/// Response from add template
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct AddTemplateResponse {
    #[serde(rename = "priTmplId")]
    pub private_template_id: String,
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

/// Category info
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryInfo {
    /// Category ID
    pub id: i32,
    /// Category name
    pub name: String,
}

/// Response from get category
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryListResponse {
    pub data: Vec<CategoryInfo>,
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PubTemplateKeywordInfo {
    #[serde(default)]
    pub kid: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub rule: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PubTemplateKeywordResponse {
    #[serde(default)]
    pub data: Vec<PubTemplateKeywordInfo>,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PubTemplateTitleInfo {
    #[serde(default)]
    pub tid: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub r#type: i32,
    #[serde(default)]
    pub category_id: i32,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PubTemplateTitleListResponse {
    #[serde(default)]
    pub data: Vec<PubTemplateTitleInfo>,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct UserNotifyRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, serde_json::Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct UserNotifyExtRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, serde_json::Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct GetUserNotifyRequest {
    #[serde(flatten)]
    pub payload: HashMap<String, serde_json::Value>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserNotifyResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Subscribe Message API
///
/// Provides methods for sending subscribe messages and managing templates.
pub struct SubscribeApi {
    context: Arc<WechatContext>,
}

impl SubscribeApi {
    /// Create a new SubscribeApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Send subscribe message
    ///
    /// POST /cgi-bin/message/subscribe/send?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `options` - Subscribe message options
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wechat_mp_sdk::api::subscribe::{
    ///     SubscribeApi, SubscribeMessageOptions, SubscribeMessageData, SubscribeMessageValue
    /// };
    ///
    /// let mut data = SubscribeMessageData::new();
    /// data.insert("thing1".to_string(), SubscribeMessageValue::new("Order #123"));
    ///
    /// let options = SubscribeMessageOptions {
    ///     touser: "user_openid".to_string(),
    ///     template_id: "template_id".to_string(),
    ///     data,
    ///     page: Some("pages/index/index".to_string()),
    ///     miniprogram_state: None,
    ///     lang: None,
    /// };
    ///
    /// subscribe_api.send(options).await?;
    /// ```
    pub async fn send(&self, options: SubscribeMessageOptions) -> Result<(), WechatError> {
        let request = SubscribeMessageRequest {
            touser: options.touser,
            template_id: options.template_id,
            page: options.page,
            data: options.data,
            miniprogram_state: options.miniprogram_state,
            lang: options.lang,
        };

        let response: SubscribeMessageResponse = self
            .context
            .authed_post("/cgi-bin/message/subscribe/send", &request)
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(())
    }

    /// Add template from template library
    ///
    /// POST /wxaapi/newtmpl/addtemplate?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `tid` - Template library ID
    /// * `kid_list` - Keyword ID list (optional)
    /// * `scene_desc` - Scene description (optional)
    ///
    /// # Returns
    /// The private template ID
    ///
    /// # Example
    ///
    /// ```ignore
    /// let pri_tmpl_id = subscribe_api.add_template("tid123", Some(vec![1, 2, 3]), Some("payment notification")).await?;
    /// println!("Template ID: {}", pri_tmpl_id);
    /// ```
    pub async fn add_template(
        &self,
        tid: &str,
        kid_list: Option<Vec<i32>>,
        scene_desc: Option<&str>,
    ) -> Result<String, WechatError> {
        let request = AddTemplateRequest {
            tid: tid.to_string(),
            kid_list,
            scene_desc: scene_desc.map(|s| s.to_string()),
        };

        let response: AddTemplateResponse = self
            .context
            .authed_post("/wxaapi/newtmpl/addtemplate", &request)
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response.private_template_id)
    }

    /// Get template list
    ///
    /// GET /wxaapi/newtmpl/gettemplate?access_token=ACCESS_TOKEN
    ///
    /// # Returns
    /// List of templates
    ///
    /// # Example
    ///
    /// ```ignore
    /// let templates = subscribe_api.get_template_list().await?;
    /// for tmpl in templates {
    ///     println!("Template: {} - {}", tmpl.private_template_id, tmpl.title);
    /// }
    /// ```
    pub async fn get_template_list(&self) -> Result<Vec<TemplateInfo>, WechatError> {
        let response: TemplateListResponse = self
            .context
            .authed_get("/wxaapi/newtmpl/gettemplate", &[])
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response.data)
    }

    /// Delete template
    ///
    /// POST /wxaapi/newtmpl/deltemplate?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `pri_tmpl_id` - Private template ID to delete
    ///
    /// # Example
    ///
    /// ```ignore
    /// subscribe_api.delete_template("pri_tmpl_id_123").await?;
    /// ```
    pub async fn delete_template(&self, pri_tmpl_id: &str) -> Result<(), WechatError> {
        #[derive(Serialize)]
        struct Request {
            #[serde(rename = "pri_tmpl_id")]
            pri_tmpl_id: String,
        }

        let response: SubscribeMessageResponse = self
            .context
            .authed_post(
                "/wxaapi/newtmpl/deltemplate",
                &Request {
                    pri_tmpl_id: pri_tmpl_id.to_string(),
                },
            )
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(())
    }

    /// Get category list
    ///
    /// GET /wxaapi/newtmpl/getcategory?access_token=ACCESS_TOKEN
    ///
    /// # Returns
    /// List of categories available for templates
    ///
    /// # Example
    ///
    /// ```ignore
    /// let categories = subscribe_api.get_category().await?;
    /// for cat in categories {
    ///     println!("Category: {} - {}", cat.id, cat.name);
    /// }
    /// ```
    pub async fn get_category(&self) -> Result<Vec<CategoryInfo>, WechatError> {
        let response: CategoryListResponse = self
            .context
            .authed_get("/wxaapi/newtmpl/getcategory", &[])
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response.data)
    }

    pub async fn get_pub_template_keywords_by_id(
        &self,
        tid: &str,
    ) -> Result<PubTemplateKeywordResponse, WechatError> {
        let response: PubTemplateKeywordResponse = self
            .context
            .authed_get("/wxaapi/newtmpl/getpubtemplatekeywords", &[("tid", tid)])
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response)
    }

    pub async fn get_pub_template_title_list(
        &self,
        ids: &[i32],
        start: i32,
        limit: i32,
    ) -> Result<PubTemplateTitleListResponse, WechatError> {
        let ids_text = ids
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>()
            .join(",");
        let start_text = start.to_string();
        let limit_text = limit.to_string();
        let response: PubTemplateTitleListResponse = self
            .context
            .authed_get(
                "/wxaapi/newtmpl/getpubtemplatetitles",
                &[
                    ("ids", ids_text.as_str()),
                    ("start", start_text.as_str()),
                    ("limit", limit_text.as_str()),
                ],
            )
            .await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response)
    }

    pub async fn set_user_notify(
        &self,
        request: &UserNotifyRequest,
    ) -> Result<UserNotifyResponse, WechatError> {
        self.post_user_notify("/cgi-bin/message/update_template_card", request)
            .await
    }

    pub async fn set_user_notify_ext(
        &self,
        request: &UserNotifyExtRequest,
    ) -> Result<UserNotifyResponse, WechatError> {
        self.post_user_notify("/cgi-bin/message/update_template_card_ext", request)
            .await
    }

    pub async fn get_user_notify(
        &self,
        request: &GetUserNotifyRequest,
    ) -> Result<UserNotifyResponse, WechatError> {
        self.post_user_notify("/cgi-bin/message/get_template_card", request)
            .await
    }

    async fn post_user_notify<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<UserNotifyResponse, WechatError> {
        let response: UserNotifyResponse = self.context.authed_post(endpoint, body).await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(response)
    }
}

impl WechatApi for SubscribeApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "subscribe"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::WechatClient;
    use crate::token::TokenManager;
    use crate::types::{AppId, AppSecret};

    fn create_test_context(base_url: &str) -> Arc<WechatContext> {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        let client = Arc::new(
            WechatClient::builder()
                .appid(appid)
                .secret(secret)
                .base_url(base_url)
                .build()
                .unwrap(),
        );
        let token_manager = Arc::new(TokenManager::new((*client).clone()));
        Arc::new(WechatContext::new(client, token_manager))
    }

    #[test]
    fn test_subscribe_message_value() {
        let value = SubscribeMessageValue::new("test value");
        assert_eq!(value.value, "test value");
    }

    #[test]
    fn test_subscribe_message_data() {
        let mut data = SubscribeMessageData::new();
        data.insert(
            "thing1".to_string(),
            SubscribeMessageValue::new("Order #123"),
        );
        data.insert(
            "time2".to_string(),
            SubscribeMessageValue::new("2024-01-01"),
        );

        assert_eq!(data.len(), 2);
        assert_eq!(data.get("thing1").unwrap().value, "Order #123");
    }

    #[test]
    fn test_subscribe_message_options() {
        let mut data = SubscribeMessageData::new();
        data.insert("key1".to_string(), SubscribeMessageValue::new("value1"));

        let options = SubscribeMessageOptions {
            touser: OpenId::new("o6_bmjrPTlm6_2sgVt7hMZOPfL2M").unwrap(),
            template_id: "template_id_456".to_string(),
            data,
            page: Some("pages/index/index".to_string()),
            miniprogram_state: Some(MiniProgramState::Developer),
            lang: Some(Lang::ZhCN),
        };

        assert_eq!(options.touser.as_str(), "o6_bmjrPTlm6_2sgVt7hMZOPfL2M");
        assert_eq!(options.template_id, "template_id_456");
        assert_eq!(options.page, Some("pages/index/index".to_string()));
    }

    #[test]
    fn test_pub_template_keywords_response_parse() {
        let json = r#"{
            "data": [{"kid": 1, "name": "thing1", "rule": "20个以内字符"}],
            "errcode": 0,
            "errmsg": "ok"
        }"#;

        let response: PubTemplateKeywordResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].kid, 1);
    }

    #[test]
    fn test_user_notify_response_parse() {
        let json = r#"{"errcode": 0, "errmsg": "ok", "status": "success"}"#;

        let response: UserNotifyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert_eq!(response.extra.get("status").unwrap(), "success");
    }

    #[tokio::test]
    async fn test_send_success() {
        use wiremock::matchers::{body_json, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/message/subscribe/send"))
            .and(query_param("access_token", "test_token"))
            .and(body_json(serde_json::json!({
                "touser": "o6_bmjrPTlm6_2sgVt7hMZOPfL2M",
                "template_id": "template_id_456",
                "data": {
                    "thing1": {"value": "Order #123"}
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/cgi-bin/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "test_token",
                "expires_in": 7200,
                "errcode": 0,
                "errmsg": ""
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let subscribe_api = SubscribeApi::new(context);

        let mut data = SubscribeMessageData::new();
        data.insert(
            "thing1".to_string(),
            SubscribeMessageValue::new("Order #123"),
        );

        let options = SubscribeMessageOptions {
            touser: OpenId::new("o6_bmjrPTlm6_2sgVt7hMZOPfL2M").unwrap(),
            template_id: "template_id_456".to_string(),
            data,
            page: None,
            miniprogram_state: None,
            lang: None,
        };

        let result = subscribe_api.send(options).await;
        assert!(result.is_ok());
    }
}
