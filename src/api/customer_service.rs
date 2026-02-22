//! Customer Service Message API
//!
//! Provides APIs for sending customer service messages to users.
//!
//! # Overview
//!
//! Customer service messages allow Mini Programs to send messages to users
//! who have interacted with the Mini Program within the last 48 hours.
//!
//! # Message Types
//!
//! - [`Message::Text`] - Text message
//! - [`Message::Image`] - Image message
//! - [`Message::Link`] - Link card message
//! - [`Message::MiniProgramPage`] - Mini Program page card
//!
//! # Example
//!
//! ```rust,ignore
//! use wechat_mp_sdk::api::customer_service::{CustomerServiceApi, Message, TextMessage};
//! use wechat_mp_sdk::token::TokenManager;
//!
//! let api = CustomerServiceApi::new(context);
//! let message = Message::Text {
//!     text: TextMessage::new("Hello!")
//! };
//! api.send("user_openid", message).await?;
//! ```

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{WechatApi, WechatContext};
use crate::error::WechatError;
use crate::types::AppId;

// ============================================================================
// Message Types
// ============================================================================

/// Message types for customer service messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "msgtype", rename_all = "lowercase")]
pub enum Message {
    /// Text message
    Text { text: TextMessage },
    /// Image message
    Image { image: MediaMessage },
    /// Link card message
    Link { link: LinkMessage },
    /// Mini Program page card
    #[serde(rename = "miniprogrampage")]
    MiniProgramPage {
        miniprogrampage: MiniProgramPageMessage,
    },
}

/// Text message content
#[derive(Debug, Clone, Serialize)]
pub struct TextMessage {
    /// Message content
    pub content: String,
}

impl TextMessage {
    /// Create a new text message
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

/// Media message (image) content
#[derive(Debug, Clone, Serialize)]
pub struct MediaMessage {
    /// Media ID from upload API
    pub media_id: String,
}

impl MediaMessage {
    /// Create a new media message
    pub fn new(media_id: impl Into<String>) -> Self {
        Self {
            media_id: media_id.into(),
        }
    }
}

/// Link message content
#[derive(Debug, Clone, Serialize)]
pub struct LinkMessage {
    /// Link title
    pub title: String,
    /// Link description
    pub description: String,
    /// Link URL
    pub url: String,
    /// Thumbnail URL
    pub thumb_url: String,
}

impl LinkMessage {
    /// Create a new link message
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

/// Mini Program page message content
#[derive(Debug, Clone, Serialize)]
pub struct MiniProgramPageMessage {
    /// Page title
    pub title: String,
    /// Mini Program AppID (can be different from current Mini Program)
    pub appid: AppId,
    /// Page path
    pub pagepath: String,
    /// Thumbnail media ID
    pub thumb_media_id: String,
}

impl MiniProgramPageMessage {
    /// Create a new Mini Program page message
    pub fn new(
        title: impl Into<String>,
        appid: AppId,
        pagepath: impl Into<String>,
        thumb_media_id: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            appid,
            pagepath: pagepath.into(),
            thumb_media_id: thumb_media_id.into(),
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request for sending customer service message
#[derive(Debug, Clone, Serialize)]
struct CustomerServiceMessageRequest {
    #[serde(rename = "touser")]
    touser: String,
    #[serde(flatten)]
    msgtype: Message,
}

#[derive(Debug, Clone, Deserialize)]
struct CustomerServiceMessageResponse {
    #[serde(default)]
    errcode: i32,
    #[serde(default)]
    errmsg: String,
}

/// Typing command for customer service
#[derive(Debug, Clone, Serialize)]
pub enum TypingCommand {
    Typing,
    CancelTyping,
}

#[derive(Debug, Clone, Serialize)]
struct SetTypingRequest {
    touser: String,
    command: TypingCommand,
}

// ============================================================================
// CustomerServiceApi
// ============================================================================

/// Customer Service Message API
///
/// Provides methods for sending customer service messages to users.
pub struct CustomerServiceApi {
    context: Arc<WechatContext>,
}

impl CustomerServiceApi {
    /// Create a new CustomerServiceApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Send customer service message
    ///
    /// POST /cgi-bin/message/custom/send?access_token=ACCESS_TOKEN
    ///
    /// # Arguments
    /// * `touser` - Recipient's OpenID
    /// * `message` - Message to send
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use wechat_mp_sdk::api::customer_service::{CustomerServiceApi, Message, TextMessage};
    ///
    /// let api = CustomerServiceApi::new(context);
    /// let message = Message::Text { text: TextMessage::new("Hello!") };
    /// api.send("user_openid", message).await?;
    /// ```
    pub async fn send(&self, touser: &str, message: Message) -> Result<(), WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(
            "/cgi-bin/message/custom/send?access_token={}",
            &access_token,
        );

        let request = CustomerServiceMessageRequest {
            touser: touser.to_string(),
            msgtype: message,
        };

        let response: CustomerServiceMessageResponse =
            self.context.client.post(&path, &request).await?;

        WechatError::check_api(response.errcode, &response.errmsg)?;

        Ok(())
    }

    /// Set typing status for customer service
    ///
    /// POST /cgi-bin/message/custom/typing?access_token=ACCESS_TOKEN
    pub async fn set_typing(
        &self,
        touser: &str,
        command: TypingCommand,
    ) -> Result<(), WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = crate::client::WechatClient::append_access_token(
            "/cgi-bin/message/custom/typing?access_token={}",
            &access_token,
        );
        let request = SetTypingRequest {
            touser: touser.to_string(),
            command,
        };
        let response: CustomerServiceMessageResponse =
            self.context.client.post(&path, &request).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(())
    }
}

impl WechatApi for CustomerServiceApi {
    fn api_name(&self) -> &'static str {
        "customer_service"
    }

    fn context(&self) -> &WechatContext {
        &self.context
    }
}

// ============================================================================
// Tests
// ============================================================================

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
        let appid = AppId::new_unchecked("wx1234567890abcdef");
        let msg = MiniProgramPageMessage::new(
            "Title",
            appid.clone(),
            "pages/index/index",
            "thumb_media_id",
        );
        assert_eq!(msg.title, "Title");
        assert_eq!(msg.appid, appid);
        assert_eq!(msg.pagepath, "pages/index/index");
        assert_eq!(msg.thumb_media_id, "thumb_media_id");
    }

    #[test]
    fn test_message_serialization() {
        let text_msg = Message::Text {
            text: TextMessage::new("Hello"),
        };
        let json = serde_json::to_string(&text_msg).unwrap();
        assert!(json.contains("\"msgtype\":\"text\""));
        assert!(json.contains("\"text\":{\"content\":\"Hello\"}"));

        let image_msg = Message::Image {
            image: MediaMessage::new("media123"),
        };
        let json = serde_json::to_string(&image_msg).unwrap();
        assert!(json.contains("\"msgtype\":\"image\""));
        assert!(json.contains("\"image\":{\"media_id\":\"media123\"}"));
    }

    #[test]
    fn test_miniprogrampage_serialization_wire_format() {
        let appid = AppId::new_unchecked("wx1234567890abcdef");
        let msg = Message::MiniProgramPage {
            miniprogrampage: MiniProgramPageMessage::new(
                "Welcome",
                appid,
                "pages/index/index",
                "thumb_media_123",
            ),
        };
        let json = serde_json::to_string(&msg).unwrap();
        // Wire format must use "miniprogrampage" (lowercase), not "MiniProgramPage"
        assert!(json.contains("\"msgtype\":\"miniprogrampage\""));
        assert!(json.contains("\"miniprogrampage\":{"));
        assert!(json.contains("\"appid\":\"wx1234567890abcdef\""));
    }

    #[test]
    fn test_api_name() {
        let context = create_test_context("http://localhost:0");
        let api = CustomerServiceApi::new(context);
        assert_eq!(api.api_name(), "customer_service");
    }

    #[test]
    fn test_typing_command_serialization() {
        let typing = serde_json::to_string(&TypingCommand::Typing).unwrap();
        assert_eq!(typing, "\"Typing\"");
        let cancel = serde_json::to_string(&TypingCommand::CancelTyping).unwrap();
        assert_eq!(cancel, "\"CancelTyping\"");
    }

    #[tokio::test]
    async fn test_send_text_message_success() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

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

        Mock::given(method("POST"))
            .and(path("/cgi-bin/message/custom/send"))
            .and(query_param("access_token", "test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = CustomerServiceApi::new(context);

        let message = Message::Text {
            text: TextMessage::new("Hello!"),
        };
        let result = api.send("test_openid", message).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_message_api_error() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

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

        Mock::given(method("POST"))
            .and(path("/cgi-bin/message/custom/send"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 40001,
                "errmsg": "invalid credential"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let api = CustomerServiceApi::new(context);

        let message = Message::Text {
            text: TextMessage::new("Hello!"),
        };
        let result = api.send("test_openid", message).await;

        assert!(result.is_err());
        if let Err(WechatError::Api { code, message }) = result {
            assert_eq!(code, 40001);
            assert_eq!(message, "invalid credential");
        } else {
            panic!("Expected WechatError::Api");
        }
    }
}
