//! Customer Service Message API
//!
//! Provides APIs for sending customer service messages and managing temporary media.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::WechatClient;
use crate::error::WechatError;
use crate::token::TokenManager;

/// Message types for customer service messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "msgtype", rename_all = "lowercase")]
pub enum Message {
    Text {
        text: TextMessage,
    },
    Image {
        image: MediaMessage,
    },
    Link {
        link: LinkMessage,
    },
    Miniprogrampage {
        miniprogrampage: MiniprogramPageMessage,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct TextMessage {
    pub content: String,
}

impl TextMessage {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaMessage {
    pub media_id: String,
}

impl MediaMessage {
    pub fn new(media_id: impl Into<String>) -> Self {
        Self {
            media_id: media_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkMessage {
    pub title: String,
    pub description: String,
    pub url: String,
    pub thumb_url: String,
}

impl LinkMessage {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        url: impl Into<String>,
        thumb_url: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            url: url.into(),
            thumb_url: thumb_url.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MiniprogramPageMessage {
    pub title: String,
    pub appid: String,
    pub pagepath: String,
    pub thumb_media_id: String,
}

impl MiniprogramPageMessage {
    pub fn new(
        title: impl Into<String>,
        appid: impl Into<String>,
        pagepath: impl Into<String>,
        thumb_media_id: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            appid: appid.into(),
            pagepath: pagepath.into(),
            thumb_media_id: thumb_media_id.into(),
        }
    }
}

/// Request for sending customer service message
#[derive(Debug, Clone, Serialize)]
struct CustomerServiceMessageRequest {
    #[serde(rename = "touser")]
    touser: String,
    #[serde(flatten)]
    msgtype: Message,
}

/// Response from customer service message API
#[derive(Debug, Clone, Deserialize)]
struct CustomerServiceMessageResponse {
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

/// Media type for upload
#[derive(Debug, Clone)]
pub enum MediaType {
    Image,
    Voice,
    Video,
    Thumb,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Image => "image",
            MediaType::Voice => "voice",
            MediaType::Video => "video",
            MediaType::Thumb => "thumb",
        }
    }
}

/// Response from media upload
#[derive(Debug, Clone, Deserialize)]
pub struct MediaUploadResponse {
    #[serde(rename = "type")]
    pub media_type: String,
    pub media_id: String,
    pub created_at: i64,
    #[serde(default)]
    pub errcode: i32,
    #[serde(default)]
    pub errmsg: String,
}

/// Message API for customer service messages and temporary media
pub struct MessageApi {
    client: WechatClient,
}

impl MessageApi {
    /// Create a new MessageApi instance
    pub fn new(client: WechatClient) -> Self {
        Self { client }
    }

    /// Send customer service message
    ///
    /// POST /cgi-bin/message/custom/send?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `token_manager` - Token manager for getting access token
    /// * `touser` - Recipient's OpenID
    /// * `message` - Message to send
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wechat_mp_sdk::api::message::{MessageApi, Message, TextMessage};
    /// use wechat_mp_sdk::token::TokenManager;
    ///
    /// let api = MessageApi::new(client);
    /// let message = Message::Text { text: TextMessage::new("Hello!") };
    /// api.send_customer_service_message(&token_manager, "user_openid", message).await?;
    /// ```
    pub async fn send_customer_service_message(
        &self,
        token_manager: &TokenManager,
        touser: &str,
        message: Message,
    ) -> Result<(), WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/cgi-bin/message/custom/send?access_token={}", access_token);

        let request = CustomerServiceMessageRequest {
            touser: touser.to_string(),
            msgtype: message,
        };

        let response: CustomerServiceMessageResponse = self.client.post(&path, &request).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(())
    }

    /// Upload temporary media (file bytes)
    ///
    /// POST /cgi-bin/media/upload?access_token=ACCESS_TOKEN&type=TYPE
    ///
    /// # Arguments
    /// * `token_manager` - Token manager for getting access token
    /// * `media_type` - Type of media (image, voice, video, thumb)
    /// * `filename` - Name of the file
    /// * `data` - File content as bytes
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wechat_mp_sdk::api::message::{MessageApi, MediaType};
    /// use wechat_mp_sdk::token::TokenManager;
    ///
    /// let api = MessageApi::new(client);
    /// let image_data = std::fs::read("image.jpg")?;
    /// let response = api.upload_temp_media(
    ///     &token_manager,
    ///     MediaType::Image,
    ///     "image.jpg",
    ///     &image_data
    /// ).await?;
    /// println!("Media ID: {}", response.media_id);
    /// ```
    pub async fn upload_temp_media(
        &self,
        token_manager: &TokenManager,
        media_type: MediaType,
        filename: &str,
        data: &[u8],
    ) -> Result<MediaUploadResponse, WechatError> {
        let access_token = token_manager.get_token().await?;
        let url = format!(
            "{}/cgi-bin/media/upload?access_token={}&type={}",
            self.client.base_url(),
            access_token,
            media_type.as_str()
        );

        let part = reqwest::multipart::Part::bytes(data.to_vec()).file_name(filename.to_string());
        let form = reqwest::multipart::Form::new().part("media", part);

        let response = self.client.http().post(&url).multipart(form).send().await?;

        let result: MediaUploadResponse = response.json().await?;

        if result.errcode != 0 {
            return Err(WechatError::Api {
                code: result.errcode,
                message: result.errmsg,
            });
        }

        Ok(result)
    }

    /// Get temporary media
    ///
    /// GET /cgi-bin/media/get?access_token=ACCESS_TOKEN&media_id=MEDIA_ID
    ///
    /// # Arguments
    /// * `token_manager` - Token manager for getting access token
    /// * `media_id` - Media ID returned from upload
    ///
    /// # Returns
    /// Raw bytes of the media file
    pub async fn get_temp_media(
        &self,
        token_manager: &TokenManager,
        media_id: &str,
    ) -> Result<Vec<u8>, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!(
            "/cgi-bin/media/get?access_token={}&media_id={}",
            access_token, media_id
        );

        let url = format!("{}{}", self.client.base_url(), path);
        let response = self.client.http().get(&url).send().await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

// ============================================================================
// Subscribe Message Types
// ============================================================================

/// Subscribe message data (key-value pairs)
pub type SubscribeMessageData = HashMap<String, SubscribeMessageValue>;

/// Value for subscribe message field
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeMessageValue {
    pub value: String,
}

impl SubscribeMessageValue {
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
    touser: String,
    #[serde(rename = "template_id")]
    template_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<String>,
    data: SubscribeMessageData,
    #[serde(skip_serializing_if = "Option::is_none")]
    miniprogram_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lang: Option<String>,
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
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateInfo {
    #[serde(rename = "pri_tmpl_id")]
    pub pri_tmpl_id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub example: Option<String>,
    #[serde(rename = "type")]
    pub template_type: i32,
}

/// Response from get template list
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
#[derive(Debug, Clone, Deserialize)]
pub struct AddTemplateResponse {
    #[serde(rename = "pri_tmpl_id")]
    pub pri_tmpl_id: String,
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

/// Category info
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryInfo {
    pub id: i32,
    pub name: String,
}

/// Response from get category
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryListResponse {
    pub data: Vec<CategoryInfo>,
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

// ============================================================================
// Subscribe Message Methods
// ============================================================================

impl MessageApi {
    /// Send subscribe message
    ///
    /// POST /cgi-bin/message/subscribe/send?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `token_manager` - Token manager for getting access token
    /// * `touser` - Recipient's OpenID
    /// * `template_id` - Template ID
    /// * `data` - Template data
    /// * `page` - Page to navigate to (optional)
    pub async fn send_subscribe_message(
        &self,
        token_manager: &TokenManager,
        touser: &str,
        template_id: &str,
        data: SubscribeMessageData,
        page: Option<&str>,
    ) -> Result<(), WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!(
            "/cgi-bin/message/subscribe/send?access_token={}",
            access_token
        );

        let request = SubscribeMessageRequest {
            touser: touser.to_string(),
            template_id: template_id.to_string(),
            page: page.map(|s| s.to_string()),
            data,
            miniprogram_state: None,
            lang: None,
        };

        let response: SubscribeMessageResponse = self.client.post(&path, &request).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(())
    }

    /// Add template from template library
    ///
    /// POST /wxaapi/newtmpl/addtemplate?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `token_manager` - Token manager for getting access token
    /// * `tid` - Template library ID
    /// * `kid_list` - Keyword ID list (optional)
    /// * `scene_desc` - Scene description (optional)
    ///
    /// # Returns
    /// The private template ID
    pub async fn add_template(
        &self,
        token_manager: &TokenManager,
        tid: &str,
        kid_list: Option<Vec<i32>>,
        scene_desc: Option<&str>,
    ) -> Result<String, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxaapi/newtmpl/addtemplate?access_token={}", access_token);

        let request = AddTemplateRequest {
            tid: tid.to_string(),
            kid_list,
            scene_desc: scene_desc.map(|s| s.to_string()),
        };

        let response: AddTemplateResponse = self.client.post(&path, &request).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(response.pri_tmpl_id)
    }

    /// Get template list
    ///
    /// GET /wxaapi/newtmpl/gettemplate?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `token_manager` - Token manager for getting access token
    ///
    /// # Returns
    /// List of templates
    pub async fn get_template_list(
        &self,
        token_manager: &TokenManager,
    ) -> Result<Vec<TemplateInfo>, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxaapi/newtmpl/gettemplate?access_token={}", access_token);

        let response: TemplateListResponse = self.client.get(&path, &[]).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(response.data)
    }

    /// Delete template
    ///
    /// POST /wxaapi/newtmpl/deltemplate?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `token_manager` - Token manager for getting access token
    /// * `pri_tmpl_id` - Private template ID to delete
    pub async fn delete_template(
        &self,
        token_manager: &TokenManager,
        pri_tmpl_id: &str,
    ) -> Result<(), WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxaapi/newtmpl/deltemplate?access_token={}", access_token);

        #[derive(Serialize)]
        struct Request {
            #[serde(rename = "pri_tmpl_id")]
            pri_tmpl_id: String,
        }

        let response: SubscribeMessageResponse = self
            .client
            .post(
                &path,
                &Request {
                    pri_tmpl_id: pri_tmpl_id.to_string(),
                },
            )
            .await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(())
    }

    /// Get category list
    ///
    /// GET /wxaapi/newtmpl/getcategory?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `token_manager` - Token manager for getting access token
    ///
    /// # Returns
    /// List of categories
    pub async fn get_category(
        &self,
        token_manager: &TokenManager,
    ) -> Result<Vec<CategoryInfo>, WechatError> {
        let access_token = token_manager.get_token().await?;
        let path = format!("/wxaapi/newtmpl/getcategory?access_token={}", access_token);

        let response: CategoryListResponse = self.client.get(&path, &[]).await?;

        if response.errcode != 0 {
            return Err(WechatError::Api {
                code: response.errcode,
                message: response.errmsg,
            });
        }

        Ok(response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AppId, AppSecret};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_client_with_base_url(base_url: &str) -> WechatClient {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .base_url(base_url)
            .build()
            .unwrap()
    }

    #[test]
    fn test_text_message() {
        let msg = TextMessage::new("Hello world");
        assert_eq!(msg.content, "Hello world");
    }

    #[test]
    fn test_media_message() {
        let msg = MediaMessage::new("media_id_123");
        assert_eq!(msg.media_id, "media_id_123");
    }

    #[test]
    fn test_link_message() {
        let msg = LinkMessage::new(
            "Title",
            "Description",
            "https://example.com",
            "https://example.com/thumb.jpg",
        );
        assert_eq!(msg.title, "Title");
        assert_eq!(msg.description, "Description");
        assert_eq!(msg.url, "https://example.com");
        assert_eq!(msg.thumb_url, "https://example.com/thumb.jpg");
    }

    #[test]
    fn test_miniprogram_page_message() {
        let msg =
            MiniprogramPageMessage::new("Title", "appid123", "pages/index/index", "thumb_media_id");
        assert_eq!(msg.title, "Title");
        assert_eq!(msg.appid, "appid123");
        assert_eq!(msg.pagepath, "pages/index/index");
        assert_eq!(msg.thumb_media_id, "thumb_media_id");
    }

    #[test]
    fn test_media_type() {
        assert_eq!(MediaType::Image.as_str(), "image");
        assert_eq!(MediaType::Voice.as_str(), "voice");
        assert_eq!(MediaType::Video.as_str(), "video");
        assert_eq!(MediaType::Thumb.as_str(), "thumb");
    }

    #[tokio::test]
    async fn test_upload_temp_media_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/media/upload"))
            .and(query_param("access_token", "test_token"))
            .and(query_param("type", "image"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "type": "image",
                "media_id": "test_media_id_123",
                "created_at": 1234567890,
                "errcode": 0,
                "errmsg": ""
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client_with_base_url(&mock_server.uri());
        let message_api = MessageApi::new(client.clone());
        let token_manager = crate::token::TokenManager::new(client);

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

        let image_data = b"fake_image_data";
        let result = message_api
            .upload_temp_media(&token_manager, MediaType::Image, "test.jpg", image_data)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.media_type, "image");
        assert_eq!(response.media_id, "test_media_id_123");
        assert_eq!(response.created_at, 1234567890);
    }

    #[tokio::test]
    async fn test_upload_temp_media_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/cgi-bin/media/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "type": "",
                "media_id": "",
                "created_at": 0,
                "errcode": 40001,
                "errmsg": "invalid credential"
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

        let client = create_test_client_with_base_url(&mock_server.uri());
        let message_api = MessageApi::new(client.clone());
        let token_manager = crate::token::TokenManager::new(client);

        let image_data = b"fake_image_data";
        let result = message_api
            .upload_temp_media(&token_manager, MediaType::Image, "test.jpg", image_data)
            .await;

        assert!(result.is_err());
        if let Err(WechatError::Api { code, message }) = result {
            assert_eq!(code, 40001);
            assert_eq!(message, "invalid credential");
        } else {
            panic!("Expected Api error");
        }
    }
}
