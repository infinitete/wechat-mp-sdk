//! Temporary Media Management API
//!
//! Provides APIs for uploading and downloading temporary media files.
//!
//! ## Overview
//!
//! WeChat Mini Programs support temporary media files that can be used in
//! customer service messages. These files are valid for 3 days after upload.
//!
//! ## Supported Media Types
//!
//! - Image (image): jpg, png formats
//! - Voice (voice): mp3, wav, amr formats
//! - Video (video): mp4 format
//! - Thumbnail (thumb): jpg, png formats
//!
//! ## Example
//!
//! ```ignore
//! use wechat_mp_sdk::api::media::{MediaApi, MediaType};
//! use wechat_mp_sdk::api::WechatContext;
//! use std::sync::Arc;
//!
//! let context = Arc::new(WechatContext::new(client, token_manager));
//! let media_api = MediaApi::new(context);
//!
//! // Upload an image
//! let image_data = std::fs::read("image.jpg")?;
//! let response = media_api.upload_temp_media(MediaType::Image, "image.jpg", &image_data).await?;
//! println!("Media ID: {}", response.media_id);
//!
//! // Download media
//! let data = media_api.get_temp_media(&response.media_id).await?;
//! ```

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::WechatError;

use super::{WechatApi, WechatContext};

/// Media type for temporary media upload
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    /// Image file (jpg, png)
    Image,
    /// Voice file (mp3, wav, amr)
    Voice,
    /// Video file (mp4)
    Video,
    /// Thumbnail file (jpg, png)
    Thumb,
}

impl MediaType {
    /// Get the string representation of the media type
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Image => "image",
            MediaType::Voice => "voice",
            MediaType::Video => "video",
            MediaType::Thumb => "thumb",
        }
    }
}

/// Response from temporary media upload
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct MediaUploadResponse {
    /// Type of the uploaded media
    #[serde(rename = "type")]
    pub media_type: String,
    /// Unique identifier for the uploaded media
    pub media_id: String,
    /// Unix timestamp when the media was created
    pub created_at: i64,
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
}

impl MediaUploadResponse {
    pub fn errcode(&self) -> i32 {
        self.errcode
    }

    pub fn errmsg(&self) -> &str {
        &self.errmsg
    }
}

/// Temporary Media API
///
/// Provides methods for uploading and downloading temporary media files.
pub struct MediaApi {
    context: Arc<WechatContext>,
}

impl MediaApi {
    /// Create a new MediaApi instance
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    /// Upload temporary media
    ///
    /// POST /cgi-bin/media/upload?access_token=ACCESS_TOKEN&type=TYPE
    ///
    /// Uploads a media file to WeChat servers. The media will be available
    /// for 3 days (72 hours) after upload.
    ///
    /// # Arguments
    /// * `media_type` - Type of media (Image, Voice, Video, Thumb)
    /// * `filename` - Name of the file (used for reference)
    /// * `data` - Raw file content as bytes
    ///
    /// # Returns
    /// `MediaUploadResponse` containing the media_id and metadata
    ///
    /// # Errors
    /// Returns `WechatError` if the upload fails or API returns an error
    ///
    /// # Example
    ///
    /// ```ignore
    /// let image_data = std::fs::read("image.jpg")?;
    /// let response = media_api.upload_temp_media(
    ///     MediaType::Image,
    ///     "image.jpg",
    ///     &image_data
    /// ).await?;
    /// println!("Media ID: {}", response.media_id);
    /// ```
    pub async fn upload_temp_media(
        &self,
        media_type: MediaType,
        filename: &str,
        data: &[u8],
    ) -> Result<MediaUploadResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let url = format!(
            "{}/cgi-bin/media/upload?access_token={}&type={}",
            self.context.client.base_url(),
            access_token,
            media_type.as_str()
        );

        let part = reqwest::multipart::Part::bytes(data.to_vec()).file_name(filename.to_string());
        let form = reqwest::multipart::Form::new().part("media", part);

        let request = self
            .context
            .client
            .http()
            .post(&url)
            .multipart(form)
            .build()?;
        let response = self.context.client.send_request(request).await?;

        let result: MediaUploadResponse = response.json().await?;

        WechatError::check_api(result.errcode, &result.errmsg)?;

        Ok(result)
    }

    /// Get temporary media
    ///
    /// GET /cgi-bin/media/get?access_token=ACCESS_TOKEN&media_id=MEDIA_ID
    ///
    /// Downloads a previously uploaded temporary media file.
    ///
    /// # Arguments
    /// * `media_id` - Media ID returned from upload_temp_media
    ///
    /// # Returns
    /// Raw bytes of the media file
    ///
    /// # Errors
    /// Returns `WechatError` if the download fails or media is not found
    ///
    /// # Example
    ///
    /// ```ignore
    /// let data = media_api.get_temp_media("media_id_123").await?;
    /// std::fs::write("downloaded.jpg", &data)?;
    /// ```
    pub async fn get_temp_media(&self, media_id: &str) -> Result<Vec<u8>, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let url = format!(
            "{}/cgi-bin/media/get?access_token={}&media_id={}",
            self.context.client.base_url(),
            access_token,
            media_id
        );

        let request = self.context.client.http().get(&url).build()?;
        let response = self.context.client.send_request(request).await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl WechatApi for MediaApi {
    fn api_name(&self) -> &'static str {
        "media"
    }

    fn context(&self) -> &WechatContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AppId, AppSecret};
    use crate::WechatClient;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
        let token_manager = Arc::new(crate::token::TokenManager::new((*client).clone()));
        Arc::new(WechatContext::new(client, token_manager))
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

        let context = create_test_context(&mock_server.uri());
        let media_api = MediaApi::new(context);

        let image_data = b"fake_image_data";
        let result = media_api
            .upload_temp_media(MediaType::Image, "test.jpg", image_data)
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

        let context = create_test_context(&mock_server.uri());
        let media_api = MediaApi::new(context);

        let image_data = b"fake_image_data";
        let result = media_api
            .upload_temp_media(MediaType::Image, "test.jpg", image_data)
            .await;

        assert!(result.is_err());
        if let Err(WechatError::Api { code, message }) = result {
            assert_eq!(code, 40001);
            assert_eq!(message, "invalid credential");
        } else {
            panic!("Expected Api error");
        }
    }

    #[tokio::test]
    async fn test_get_temp_media_success() {
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

        Mock::given(method("GET"))
            .and(path("/cgi-bin/media/get"))
            .and(query_param("access_token", "test_token"))
            .and(query_param("media_id", "test_media_id"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(b"media_binary_data", "image/jpeg"),
            )
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let media_api = MediaApi::new(context);

        let result = media_api.get_temp_media("test_media_id").await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data, b"media_binary_data");
    }

    #[tokio::test]
    async fn test_get_temp_media_error_json() {
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

        Mock::given(method("GET"))
            .and(path("/cgi-bin/media/get"))
            .and(query_param("access_token", "test_token"))
            .and(query_param("media_id", "expired_media"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errcode": 40007,
                "errmsg": "invalid media_id"
            })))
            .mount(&mock_server)
            .await;

        let context = create_test_context(&mock_server.uri());
        let media_api = MediaApi::new(context);

        let result = media_api.get_temp_media("expired_media").await;

        assert!(result.is_ok());
        let data = result.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&data).unwrap();
        assert_eq!(body["errcode"], 40007);
    }
}
