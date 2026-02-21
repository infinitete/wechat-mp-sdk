//! Unified WeChat Mini Program SDK client

use std::sync::Arc;

use crate::api::auth::LoginResponse;
use crate::api::qrcode::{
    QrcodeApi, QrcodeOptions, ShortLinkOptions, UnlimitQrcodeOptions, UrlLinkOptions,
    UrlSchemeOptions,
};
use crate::api::subscribe::SubscribeApi;
use crate::api::template::TemplateApi;
use crate::api::user::PhoneNumberResponse;
use crate::api::WechatContext;
use crate::api::{
    CategoryInfo, MediaApi, MediaType, MediaUploadResponse, Message, SubscribeMessageOptions,
    TemplateInfo,
};
use crate::crypto::{decrypt_user_data, verify_watermark, DecryptedUserData};
use crate::error::WechatError;
use crate::types::{AppId, SessionKey};

/// Unified WeChat Mini Program client
///
/// This is the main entry point for the SDK. It provides access to all
/// WeChat APIs through a unified interface.
///
/// # Example
///
/// ```rust,ignore
/// use wechat_mp_sdk::WechatMp;
/// use wechat_mp_sdk::types::{AppId, AppSecret};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let wechat = WechatMp::builder()
///         .appid(AppId::new("wx1234567890abcdef")?)
///         .secret(AppSecret::new("your_secret")?)
///         .build()?;
///
///     let login_response = wechat.auth_login("code").await?;
///     println!("OpenID: {}", login_response.openid);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct WechatMp {
    context: Arc<WechatContext>,
    appid: AppId,
}

impl WechatMp {
    pub fn builder() -> super::builder::WechatMpBuilder {
        super::builder::WechatMpBuilder::default()
    }

    pub fn appid(&self) -> &str {
        self.appid.as_str()
    }

    pub async fn get_access_token(&self) -> Result<String, WechatError> {
        self.context.token_manager.get_token().await
    }

    pub async fn invalidate_token(&self) {
        self.context.token_manager.invalidate().await;
    }

    // Auth API

    pub async fn auth_login(&self, js_code: &str) -> Result<LoginResponse, WechatError> {
        crate::api::auth::AuthApi::new(self.context.clone())
            .login(js_code)
            .await
    }

    // User API

    pub async fn get_phone_number(&self, code: &str) -> Result<PhoneNumberResponse, WechatError> {
        crate::api::user::UserApi::new(self.context.clone())
            .get_phone_number(code)
            .await
    }

    // Message API

    pub async fn send_customer_service_message(
        &self,
        touser: &str,
        message: Message,
    ) -> Result<(), WechatError> {
        crate::api::customer_service::CustomerServiceApi::new(self.context.clone())
            .send(touser, message)
            .await
    }

    pub async fn upload_temp_media(
        &self,
        media_type: MediaType,
        filename: &str,
        data: &[u8],
    ) -> Result<MediaUploadResponse, WechatError> {
        MediaApi::new(self.context.clone())
            .upload_temp_media(media_type, filename, data)
            .await
    }

    pub async fn get_temp_media(&self, media_id: &str) -> Result<Vec<u8>, WechatError> {
        MediaApi::new(self.context.clone())
            .get_temp_media(media_id)
            .await
    }

    pub async fn send_subscribe_message(
        &self,
        options: SubscribeMessageOptions,
    ) -> Result<(), WechatError> {
        SubscribeApi::new(self.context.clone()).send(options).await
    }

    pub async fn add_template(
        &self,
        tid: &str,
        kid_list: Option<Vec<i32>>,
        scene_desc: Option<&str>,
    ) -> Result<String, WechatError> {
        TemplateApi::new(self.context.clone())
            .add_template(tid, kid_list, scene_desc)
            .await
    }

    pub async fn get_template_list(&self) -> Result<Vec<TemplateInfo>, WechatError> {
        TemplateApi::new(self.context.clone())
            .get_template_list()
            .await
    }

    pub async fn delete_template(&self, pri_tmpl_id: &str) -> Result<(), WechatError> {
        TemplateApi::new(self.context.clone())
            .delete_template(pri_tmpl_id)
            .await
    }

    pub async fn get_category(&self) -> Result<Vec<CategoryInfo>, WechatError> {
        TemplateApi::new(self.context.clone()).get_category().await
    }

    // QR Code API

    pub async fn get_wxa_code(&self, options: QrcodeOptions) -> Result<Vec<u8>, WechatError> {
        QrcodeApi::new(self.context.clone())
            .get_wxa_code(options)
            .await
    }

    pub async fn get_wxa_code_unlimit(
        &self,
        options: UnlimitQrcodeOptions,
    ) -> Result<Vec<u8>, WechatError> {
        QrcodeApi::new(self.context.clone())
            .get_wxa_code_unlimit(options)
            .await
    }

    pub async fn create_qrcode(
        &self,
        path: &str,
        width: Option<u32>,
    ) -> Result<Vec<u8>, WechatError> {
        QrcodeApi::new(self.context.clone())
            .create_qrcode(path, width)
            .await
    }

    pub async fn generate_url_scheme(
        &self,
        options: UrlSchemeOptions,
    ) -> Result<String, WechatError> {
        QrcodeApi::new(self.context.clone())
            .generate_url_scheme(options)
            .await
    }

    pub async fn generate_url_link(&self, options: UrlLinkOptions) -> Result<String, WechatError> {
        QrcodeApi::new(self.context.clone())
            .generate_url_link(options)
            .await
    }

    pub async fn generate_short_link(
        &self,
        options: ShortLinkOptions,
    ) -> Result<String, WechatError> {
        QrcodeApi::new(self.context.clone())
            .generate_short_link(options)
            .await
    }

    // Crypto API

    pub fn decrypt_user_data(
        &self,
        session_key: &SessionKey,
        encrypted_data: &str,
        iv: &str,
    ) -> Result<DecryptedUserData, WechatError> {
        decrypt_user_data(session_key.as_str(), encrypted_data, iv)
    }

    pub fn verify_watermark(&self, data: &DecryptedUserData) -> Result<(), WechatError> {
        verify_watermark(data, self.appid.as_str())
    }
}

impl From<Arc<WechatContext>> for WechatMp {
    fn from(context: Arc<WechatContext>) -> Self {
        let appid = AppId::new_unchecked(context.client.appid());
        Self { context, appid }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::token::CachedToken;
    use crate::types::{AccessToken, AppSecret, Watermark};

    fn create_test_wechat_mp() -> WechatMp {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        WechatMp::builder()
            .appid(appid)
            .secret(secret)
            .build()
            .unwrap()
    }

    #[test]
    fn test_wechat_mp_decrypt_user_data() {
        let wechat = create_test_wechat_mp();
        let session_key = SessionKey::new("MTIzNDU2Nzg5MDEyMzQ1Ng==").unwrap();
        let encrypted_data = "not-valid-base64!!!";
        let iv = "MTIzNDU2Nzg5MDEyMzQ1Ng==";

        let facade_result = wechat.decrypt_user_data(&session_key, encrypted_data, iv);
        let direct_result =
            crate::crypto::aes::decrypt_user_data(session_key.as_str(), encrypted_data, iv);

        assert!(facade_result.is_err());
        assert!(direct_result.is_err());
        match facade_result.unwrap_err() {
            WechatError::Crypto(msg) => assert!(msg.contains("Invalid encrypted_data")),
            err => panic!("Expected crypto error, got {:?}", err),
        }
    }

    #[test]
    fn test_wechat_mp_verify_watermark() {
        let wechat = create_test_wechat_mp();

        let valid_data = DecryptedUserData::new(
            serde_json::json!({"openId": "o123"}),
            Watermark::new(1_700_000_000, wechat.appid()),
        );
        assert!(wechat.verify_watermark(&valid_data).is_ok());

        let invalid_data = DecryptedUserData::new(
            serde_json::json!({"openId": "o123"}),
            Watermark::new(1_700_000_000, "wx0000000000000000"),
        );
        match wechat.verify_watermark(&invalid_data).unwrap_err() {
            WechatError::Signature(msg) => assert!(msg.contains("Watermark appid mismatch")),
            err => panic!("Expected signature error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_wechat_mp_get_access_token() {
        let wechat = create_test_wechat_mp();
        let cached = CachedToken {
            token: AccessToken::new("cached_access_token").unwrap(),
            expires_at: Instant::now() + Duration::from_secs(7200),
        };
        *wechat.context.token_manager.cache.write().await = Some(cached);

        let token = wechat.get_access_token().await.unwrap();

        assert_eq!(token, "cached_access_token");
    }

    #[tokio::test]
    async fn test_wechat_mp_invalidate_token() {
        let wechat = create_test_wechat_mp();
        let cached = CachedToken {
            token: AccessToken::new("cached_access_token").unwrap(),
            expires_at: Instant::now() + Duration::from_secs(7200),
        };
        *wechat.context.token_manager.cache.write().await = Some(cached);

        wechat.invalidate_token().await;

        assert!(wechat.context.token_manager.cache.read().await.is_none());
    }
}
