//! Template Message Management API
//!
//! Provides APIs for managing WeChat Mini Program template messages.
//! Delegates to [`super::subscribe::SubscribeApi`] for the underlying implementation.
//!
//! # Features
//!
//! - Add templates from template library
//! - Get template list
//! - Delete templates
//! - Get category list
//!
//! # Example
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use wechat_mp_sdk::api::template::TemplateApi;
//! # use wechat_mp_sdk::api::WechatContext;
//! # use wechat_mp_sdk::client::WechatClient;
//! # use wechat_mp_sdk::token::TokenManager;
//! # use wechat_mp_sdk::types::{AppId, AppSecret};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = Arc::new(WechatClient::builder()
//! #     .appid(AppId::new("wx1234567890abcdef")?)
//! #     .secret(AppSecret::new("test_secret")?)
//! #     .build()?);
//! # let token_manager = Arc::new(TokenManager::new((*client).clone()));
//! # let context = Arc::new(WechatContext::new(client, token_manager));
//! let template_api = TemplateApi::new(context);
//!
//! // Get template list
//! let templates = template_api.get_template_list().await?;
//! for tmpl in templates {
//!     println!("Template: {} - {}", tmpl.private_template_id, tmpl.title);
//! }
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use super::subscribe::SubscribeApi;
use super::{WechatApi, WechatContext};
use crate::error::WechatError;

// Re-export shared types from subscribe module for backward compatibility
pub use super::subscribe::{
    AddTemplateResponse, CategoryInfo, CategoryListResponse, TemplateInfo, TemplateListResponse,
};

/// Template Message Management API
///
/// Thin wrapper around [`SubscribeApi`] for template management operations.
/// Subscribe messages and template management share the same underlying WeChat APIs.
pub struct TemplateApi {
    subscribe_api: SubscribeApi,
}

impl TemplateApi {
    /// Create a new TemplateApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self {
            subscribe_api: SubscribeApi::new(context),
        }
    }

    /// Add template from template library
    ///
    /// Delegates to [`SubscribeApi::add_template`].
    pub async fn add_template(
        &self,
        tid: &str,
        kid_list: Option<Vec<i32>>,
        scene_desc: Option<&str>,
    ) -> Result<String, WechatError> {
        self.subscribe_api
            .add_template(tid, kid_list, scene_desc)
            .await
    }

    /// Get template list
    ///
    /// Delegates to [`SubscribeApi::get_template_list`].
    pub async fn get_template_list(&self) -> Result<Vec<TemplateInfo>, WechatError> {
        self.subscribe_api.get_template_list().await
    }

    /// Delete template
    ///
    /// Delegates to [`SubscribeApi::delete_template`].
    pub async fn delete_template(&self, pri_tmpl_id: &str) -> Result<(), WechatError> {
        self.subscribe_api.delete_template(pri_tmpl_id).await
    }

    /// Get category list
    ///
    /// Delegates to [`SubscribeApi::get_category`].
    pub async fn get_category(&self) -> Result<Vec<CategoryInfo>, WechatError> {
        self.subscribe_api.get_category().await
    }
}

impl WechatApi for TemplateApi {
    fn context(&self) -> &WechatContext {
        self.subscribe_api.context()
    }

    fn api_name(&self) -> &'static str {
        "template"
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
    fn test_template_info_deserialize() {
        let json = serde_json::json!({
            "priTmplId": "test_template_id",
            "title": "Test Template",
            "content": "Content here",
            "example": "Example content",
            "type": 2
        });

        let info: TemplateInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.private_template_id, "test_template_id");
        assert_eq!(info.title, "Test Template");
        assert_eq!(info.content, "Content here");
        assert_eq!(info.example, Some("Example content".to_string()));
        assert_eq!(info.template_type, 2);
    }

    #[test]
    fn test_template_info_without_example() {
        let json = serde_json::json!({
            "priTmplId": "test_template_id",
            "title": "Test Template",
            "content": "Content here",
            "type": 2
        });

        let info: TemplateInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.example, None);
    }

    #[test]
    fn test_category_info_deserialize() {
        let json = serde_json::json!({
            "id": 123,
            "name": "Category Name"
        });

        let info: CategoryInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.id, 123);
        assert_eq!(info.name, "Category Name");
    }

    #[tokio::test]
    async fn test_get_template_list_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/wxaapi/newtmpl/gettemplate"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "priTmplId": "template_id_1",
                        "title": "Purchase Notification",
                        "content": "Purchase: {{thing1.DATA}}",
                        "type": 2
                    }
                ],
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
        let template_api = TemplateApi::new(context);

        let result = template_api.get_template_list().await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].private_template_id, "template_id_1");
    }

    #[tokio::test]
    async fn test_add_template_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/wxaapi/newtmpl/addtemplate"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "priTmplId": "new_private_template_id",
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
        let template_api = TemplateApi::new(context);

        let result = template_api
            .add_template("AA1234", Some(vec![1, 2, 3]), Some("test scene"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "new_private_template_id");
    }

    #[tokio::test]
    async fn test_delete_template_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/wxaapi/newtmpl/deltemplate"))
            .and(query_param("access_token", "test_token"))
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
        let template_api = TemplateApi::new(context);

        let result = template_api.delete_template("template_to_delete").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_category_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/wxaapi/newtmpl/getcategory"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"id": 1, "name": "IT Technology"},
                    {"id": 2, "name": "E-commerce"}
                ],
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
        let template_api = TemplateApi::new(context);

        let result = template_api.get_category().await;

        assert!(result.is_ok());
        let categories = result.unwrap();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].name, "IT Technology");
        assert_eq!(categories[1].name, "E-commerce");
    }
}
