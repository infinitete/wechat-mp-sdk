//! Unified WeChat Mini Program SDK client

use std::sync::Arc;

use crate::api::advertising::{AdvertisingApi, AdvertisingRequest, AdvertisingResponse};
use crate::api::analytics::{
    AnalyticsApi, AnalyticsDateRangeRequest, AnalyticsResponse, PerformanceDataRequest,
};
use crate::api::auth::{LoginResponse, ResetSessionKeyResponse, StableAccessTokenResponse};
use crate::api::cloud::{
    CloudApi, CloudDatabaseRequest, CloudResponse, DelayedFunctionTaskRequest,
    DeleteCloudFileRequest, DownloadFileLinkRequest, InvokeCloudFunctionRequest,
    SendCloudBaseSmsRequest, UploadFileLinkRequest,
};
use crate::api::customer_service::TypingCommand;
use crate::api::delivery::{DeliveryApi, DeliveryRequest, DeliveryResponse};
use crate::api::face::{FaceApi, FaceResponse, GetVerifyIdRequest, QueryVerifyInfoRequest};
use crate::api::hardware::{HardwareApi, HardwareRequest, HardwareResponse};
use crate::api::live::{DeleteRoomRequest, GetLiveInfoRequest, LiveApi, LiveRequest, LiveResponse};
use crate::api::logistics::{LogisticsApi, LogisticsRequest, LogisticsResponse};
use crate::api::nearby::{
    AddNearbyPoiRequest, DeleteNearbyPoiRequest, NearbyApi, NearbyPoiListRequest, NearbyResponse,
    NearbyShowStatusRequest,
};
use crate::api::ocr::{IdCardOcrRequest, OcrApi, OcrImageRequest, OcrResponse};
use crate::api::openapi::{
    ApiQuotaResponse, CallbackCheckResponse, IpListResponse, OpenApiApi, RidInfoResponse,
};
use crate::api::operations::{
    FeedbackMediaRequest, FeedbackRequest, JsErrDetailRequest, JsErrListRequest, OperationsApi,
    OperationsResponse, RealtimeLogSearchRequest,
};
use crate::api::plugin::{
    ManagePluginApplicationRequest, ManagePluginRequest, PluginApi, PluginResponse,
};
use crate::api::qrcode::{
    NfcSchemeOptions, NfcSchemeResponse, QrcodeApi, QrcodeOptions, QuerySchemeResponse,
    QueryUrlLinkResponse, ShortLinkOptions, UnlimitQrcodeOptions, UrlLinkOptions, UrlSchemeOptions,
};
use crate::api::security::{
    MediaCheckAsyncResponse, MsgSecCheckResponse, SecurityApi, UserRiskRankOptions,
    UserRiskRankResponse,
};
use crate::api::service_market::{InvokeServiceRequest, ServiceMarketApi, ServiceMarketResponse};
use crate::api::soter::{SoterApi, VerifySignatureRequest, VerifySignatureResponse};
use crate::api::subscribe::SubscribeApi;
use crate::api::subscribe::{
    GetUserNotifyRequest, PubTemplateKeywordResponse, PubTemplateTitleListResponse,
    UserNotifyExtRequest, UserNotifyRequest, UserNotifyResponse,
};
use crate::api::template::TemplateApi;
use crate::api::user::{
    CheckEncryptedDataResponse, PaidUnionIdResponse, PhoneNumberResponse, PluginOpenPIdResponse,
    UserEncryptKeyResponse,
};
use crate::api::wechat_kf::{KfWorkBoundResponse, WechatKfApi};
use crate::api::wxsearch::{SubmitPagesRequest, SubmitPagesResponse, WxsearchApi};
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

    pub async fn get_stable_access_token(
        &self,
        force_refresh: bool,
    ) -> Result<StableAccessTokenResponse, WechatError> {
        crate::api::auth::AuthApi::new(self.context.clone())
            .get_stable_access_token(force_refresh)
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

    // OpenApi Management

    pub async fn clear_quota(&self) -> Result<(), WechatError> {
        OpenApiApi::new(self.context.clone()).clear_quota().await
    }

    pub async fn get_api_quota(&self, cgi_path: &str) -> Result<ApiQuotaResponse, WechatError> {
        OpenApiApi::new(self.context.clone())
            .get_api_quota(cgi_path)
            .await
    }

    pub async fn clear_api_quota(&self, cgi_path: &str) -> Result<(), WechatError> {
        OpenApiApi::new(self.context.clone())
            .clear_api_quota(cgi_path)
            .await
    }

    pub async fn clear_quota_by_app_secret(&self) -> Result<(), WechatError> {
        OpenApiApi::new(self.context.clone())
            .clear_quota_by_app_secret()
            .await
    }

    pub async fn get_rid_info(&self, rid: &str) -> Result<RidInfoResponse, WechatError> {
        OpenApiApi::new(self.context.clone())
            .get_rid_info(rid)
            .await
    }

    pub async fn callback_check(
        &self,
        action: &str,
        check_operator: &str,
    ) -> Result<CallbackCheckResponse, WechatError> {
        OpenApiApi::new(self.context.clone())
            .callback_check(action, check_operator)
            .await
    }

    pub async fn get_api_domain_ip(&self) -> Result<IpListResponse, WechatError> {
        OpenApiApi::new(self.context.clone())
            .get_api_domain_ip()
            .await
    }

    pub async fn get_callback_ip(&self) -> Result<IpListResponse, WechatError> {
        OpenApiApi::new(self.context.clone())
            .get_callback_ip()
            .await
    }

    // Security API

    pub async fn msg_sec_check(
        &self,
        openid: &str,
        scene: u8,
        content: &str,
    ) -> Result<MsgSecCheckResponse, WechatError> {
        SecurityApi::new(self.context.clone())
            .msg_sec_check(openid, scene, content)
            .await
    }

    pub async fn media_check_async(
        &self,
        media_url: &str,
        media_type: u8,
        openid: &str,
        scene: u8,
    ) -> Result<MediaCheckAsyncResponse, WechatError> {
        SecurityApi::new(self.context.clone())
            .media_check_async(media_url, media_type, openid, scene)
            .await
    }

    pub async fn get_user_risk_rank(
        &self,
        openid: &str,
        scene: u8,
        options: Option<UserRiskRankOptions>,
    ) -> Result<UserRiskRankResponse, WechatError> {
        SecurityApi::new(self.context.clone())
            .get_user_risk_rank(openid, scene, options)
            .await
    }

    // Auth Extensions

    pub async fn check_session_key(
        &self,
        openid: &str,
        signature: &str,
        sig_method: &str,
    ) -> Result<(), WechatError> {
        crate::api::auth::AuthApi::new(self.context.clone())
            .check_session_key(openid, signature, sig_method)
            .await
    }

    pub async fn reset_user_session_key(
        &self,
        openid: &str,
        signature: &str,
        sig_method: &str,
    ) -> Result<ResetSessionKeyResponse, WechatError> {
        crate::api::auth::AuthApi::new(self.context.clone())
            .reset_user_session_key(openid, signature, sig_method)
            .await
    }

    // User Extensions

    pub async fn get_plugin_open_pid(
        &self,
        code: &str,
    ) -> Result<PluginOpenPIdResponse, WechatError> {
        crate::api::user::UserApi::new(self.context.clone())
            .get_plugin_open_pid(code)
            .await
    }

    pub async fn check_encrypted_data(
        &self,
        encrypted_msg_hash: &str,
    ) -> Result<CheckEncryptedDataResponse, WechatError> {
        crate::api::user::UserApi::new(self.context.clone())
            .check_encrypted_data(encrypted_msg_hash)
            .await
    }

    pub async fn get_paid_unionid(
        &self,
        openid: &str,
        transaction_id: &str,
    ) -> Result<PaidUnionIdResponse, WechatError> {
        crate::api::user::UserApi::new(self.context.clone())
            .get_paid_unionid(openid, transaction_id)
            .await
    }

    pub async fn get_user_encrypt_key(
        &self,
        openid: &str,
        signature: &str,
        sig_method: &str,
    ) -> Result<UserEncryptKeyResponse, WechatError> {
        crate::api::user::UserApi::new(self.context.clone())
            .get_user_encrypt_key(openid, signature, sig_method)
            .await
    }

    // QR Code Extensions

    pub async fn query_scheme(&self, scheme: &str) -> Result<QuerySchemeResponse, WechatError> {
        QrcodeApi::new(self.context.clone())
            .query_scheme(scheme)
            .await
    }

    pub async fn query_url_link(
        &self,
        url_link: &str,
    ) -> Result<QueryUrlLinkResponse, WechatError> {
        QrcodeApi::new(self.context.clone())
            .query_url_link(url_link)
            .await
    }

    pub async fn generate_nfc_scheme(
        &self,
        options: NfcSchemeOptions,
    ) -> Result<NfcSchemeResponse, WechatError> {
        QrcodeApi::new(self.context.clone())
            .generate_nfc_scheme(options)
            .await
    }

    // Customer Service Extensions

    pub async fn set_typing(
        &self,
        touser: &str,
        command: TypingCommand,
    ) -> Result<(), WechatError> {
        crate::api::customer_service::CustomerServiceApi::new(self.context.clone())
            .set_typing(touser, command)
            .await
    }

    // WeChat KF API

    pub async fn get_kf_work_bound(
        &self,
        openid: &str,
    ) -> Result<KfWorkBoundResponse, WechatError> {
        WechatKfApi::new(self.context.clone())
            .get_kf_work_bound(openid)
            .await
    }

    pub async fn bind_kf_work(&self, openid: &str, open_kfid: &str) -> Result<(), WechatError> {
        WechatKfApi::new(self.context.clone())
            .bind_kf_work(openid, open_kfid)
            .await
    }

    pub async fn unbind_kf_work(&self, openid: &str, open_kfid: &str) -> Result<(), WechatError> {
        WechatKfApi::new(self.context.clone())
            .unbind_kf_work(openid, open_kfid)
            .await
    }

    pub async fn get_pub_template_keywords_by_id(
        &self,
        tid: &str,
    ) -> Result<PubTemplateKeywordResponse, WechatError> {
        SubscribeApi::new(self.context.clone())
            .get_pub_template_keywords_by_id(tid)
            .await
    }

    pub async fn get_pub_template_title_list(
        &self,
        ids: &[i32],
        start: i32,
        limit: i32,
    ) -> Result<PubTemplateTitleListResponse, WechatError> {
        SubscribeApi::new(self.context.clone())
            .get_pub_template_title_list(ids, start, limit)
            .await
    }

    pub async fn set_user_notify(
        &self,
        request: &UserNotifyRequest,
    ) -> Result<UserNotifyResponse, WechatError> {
        SubscribeApi::new(self.context.clone())
            .set_user_notify(request)
            .await
    }

    pub async fn set_user_notify_ext(
        &self,
        request: &UserNotifyExtRequest,
    ) -> Result<UserNotifyResponse, WechatError> {
        SubscribeApi::new(self.context.clone())
            .set_user_notify_ext(request)
            .await
    }

    pub async fn get_user_notify(
        &self,
        request: &GetUserNotifyRequest,
    ) -> Result<UserNotifyResponse, WechatError> {
        SubscribeApi::new(self.context.clone())
            .get_user_notify(request)
            .await
    }

    pub async fn get_daily_summary(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_daily_summary(request)
            .await
    }

    pub async fn get_daily_visit_trend(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_daily_visit_trend(request)
            .await
    }

    pub async fn get_weekly_visit_trend(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_weekly_visit_trend(request)
            .await
    }

    pub async fn get_monthly_visit_trend(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_monthly_visit_trend(request)
            .await
    }

    pub async fn get_daily_retain(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_daily_retain(request)
            .await
    }

    pub async fn get_weekly_retain(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_weekly_retain(request)
            .await
    }

    pub async fn get_monthly_retain(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_monthly_retain(request)
            .await
    }

    pub async fn get_visit_page(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_visit_page(request)
            .await
    }

    pub async fn get_visit_distribution(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_visit_distribution(request)
            .await
    }

    pub async fn get_user_portrait(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_user_portrait(request)
            .await
    }

    pub async fn get_performance_data(
        &self,
        request: &PerformanceDataRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        AnalyticsApi::new(self.context.clone())
            .get_performance_data(request)
            .await
    }

    pub async fn get_domain_info(&self) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_domain_info()
            .await
    }

    pub async fn get_operations_performance(&self) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_performance()
            .await
    }

    pub async fn get_scene_list(&self) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_scene_list()
            .await
    }

    pub async fn get_version_list(&self) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_version_list()
            .await
    }

    pub async fn realtime_log_search(
        &self,
        request: &RealtimeLogSearchRequest,
    ) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .realtime_log_search(request)
            .await
    }

    pub async fn get_feedback(
        &self,
        request: &FeedbackRequest,
    ) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_feedback(request)
            .await
    }

    pub async fn get_feedback_media(
        &self,
        request: &FeedbackMediaRequest,
    ) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_feedback_media(request)
            .await
    }

    pub async fn get_js_err_detail(
        &self,
        request: &JsErrDetailRequest,
    ) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_js_err_detail(request)
            .await
    }

    pub async fn get_js_err_list(
        &self,
        request: &JsErrListRequest,
    ) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_js_err_list(request)
            .await
    }

    pub async fn get_gray_release_plan(&self) -> Result<OperationsResponse, WechatError> {
        OperationsApi::new(self.context.clone())
            .get_gray_release_plan()
            .await
    }

    pub async fn manage_plugin_application(
        &self,
        request: &ManagePluginApplicationRequest,
    ) -> Result<PluginResponse, WechatError> {
        PluginApi::new(self.context.clone())
            .manage_plugin_application(request)
            .await
    }

    pub async fn manage_plugin(
        &self,
        request: &ManagePluginRequest,
    ) -> Result<PluginResponse, WechatError> {
        PluginApi::new(self.context.clone())
            .manage_plugin(request)
            .await
    }

    pub async fn add_nearby_poi(
        &self,
        request: &AddNearbyPoiRequest,
    ) -> Result<NearbyResponse, WechatError> {
        NearbyApi::new(self.context.clone())
            .add_nearby_poi(request)
            .await
    }

    pub async fn delete_nearby_poi(
        &self,
        request: &DeleteNearbyPoiRequest,
    ) -> Result<NearbyResponse, WechatError> {
        NearbyApi::new(self.context.clone())
            .delete_nearby_poi(request)
            .await
    }

    pub async fn get_nearby_poi_list(
        &self,
        request: &NearbyPoiListRequest,
    ) -> Result<NearbyResponse, WechatError> {
        NearbyApi::new(self.context.clone())
            .get_nearby_poi_list(request)
            .await
    }

    pub async fn set_nearby_show_status(
        &self,
        request: &NearbyShowStatusRequest,
    ) -> Result<NearbyResponse, WechatError> {
        NearbyApi::new(self.context.clone())
            .set_show_status(request)
            .await
    }

    pub async fn invoke_cloud_function(
        &self,
        request: &InvokeCloudFunctionRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .invoke_cloud_function(request)
            .await
    }

    pub async fn add_delayed_function_task(
        &self,
        request: &DelayedFunctionTaskRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .add_delayed_function_task(request)
            .await
    }

    pub async fn database_add(
        &self,
        request: &CloudDatabaseRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .database_add(request)
            .await
    }

    pub async fn database_delete(
        &self,
        request: &CloudDatabaseRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .database_delete(request)
            .await
    }

    pub async fn database_update(
        &self,
        request: &CloudDatabaseRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .database_update(request)
            .await
    }

    pub async fn database_query(
        &self,
        request: &CloudDatabaseRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .database_query(request)
            .await
    }

    pub async fn get_upload_file_link(
        &self,
        request: &UploadFileLinkRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .get_upload_file_link(request)
            .await
    }

    pub async fn get_download_file_link(
        &self,
        request: &DownloadFileLinkRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .get_download_file_link(request)
            .await
    }

    pub async fn delete_cloud_file(
        &self,
        request: &DeleteCloudFileRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .delete_cloud_file(request)
            .await
    }

    pub async fn new_send_cloud_base_sms(
        &self,
        request: &SendCloudBaseSmsRequest,
    ) -> Result<CloudResponse, WechatError> {
        CloudApi::new(self.context.clone())
            .new_send_cloud_base_sms(request)
            .await
    }

    pub async fn create_room(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone())
            .create_room(request)
            .await
    }

    pub async fn delete_room(
        &self,
        request: &DeleteRoomRequest,
    ) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone())
            .delete_room(request)
            .await
    }

    pub async fn edit_room(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone()).edit_room(request).await
    }

    pub async fn get_live_info(
        &self,
        request: &GetLiveInfoRequest,
    ) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone())
            .get_live_info(request)
            .await
    }

    pub async fn add_goods(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone()).add_goods(request).await
    }

    pub async fn update_goods_info(
        &self,
        request: &LiveRequest,
    ) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone())
            .update_goods_info(request)
            .await
    }

    pub async fn delete_goods_info(
        &self,
        request: &LiveRequest,
    ) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone())
            .delete_goods_info(request)
            .await
    }

    pub async fn push_message(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone())
            .push_message(request)
            .await
    }

    pub async fn get_followers(&self, request: &LiveRequest) -> Result<LiveResponse, WechatError> {
        LiveApi::new(self.context.clone())
            .get_followers(request)
            .await
    }

    pub async fn send_hardware_device_message(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        HardwareApi::new(self.context.clone())
            .send_hardware_device_message(request)
            .await
    }

    pub async fn get_sn_ticket(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        HardwareApi::new(self.context.clone())
            .get_sn_ticket(request)
            .await
    }

    pub async fn create_iot_group_id(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        HardwareApi::new(self.context.clone())
            .create_iot_group_id(request)
            .await
    }

    pub async fn get_iot_group_info(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        HardwareApi::new(self.context.clone())
            .get_iot_group_info(request)
            .await
    }

    pub async fn add_iot_group_device(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        HardwareApi::new(self.context.clone())
            .add_iot_group_device(request)
            .await
    }

    pub async fn remove_iot_group_device(
        &self,
        request: &HardwareRequest,
    ) -> Result<HardwareResponse, WechatError> {
        HardwareApi::new(self.context.clone())
            .remove_iot_group_device(request)
            .await
    }

    pub async fn ai_crop(&self, request: &OcrImageRequest) -> Result<OcrResponse, WechatError> {
        OcrApi::new(self.context.clone()).ai_crop(request).await
    }

    pub async fn scan_qr_code(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        OcrApi::new(self.context.clone())
            .scan_qr_code(request)
            .await
    }

    pub async fn printed_text_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        OcrApi::new(self.context.clone())
            .printed_text_ocr(request)
            .await
    }

    pub async fn vehicle_license_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        OcrApi::new(self.context.clone())
            .vehicle_license_ocr(request)
            .await
    }

    pub async fn bank_card_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        OcrApi::new(self.context.clone())
            .bank_card_ocr(request)
            .await
    }

    pub async fn business_license_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        OcrApi::new(self.context.clone())
            .business_license_ocr(request)
            .await
    }

    pub async fn driver_license_ocr(
        &self,
        request: &OcrImageRequest,
    ) -> Result<OcrResponse, WechatError> {
        OcrApi::new(self.context.clone())
            .driver_license_ocr(request)
            .await
    }

    pub async fn id_card_ocr(
        &self,
        request: &IdCardOcrRequest,
    ) -> Result<OcrResponse, WechatError> {
        OcrApi::new(self.context.clone()).id_card_ocr(request).await
    }

    pub async fn get_all_imme_delivery(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        DeliveryApi::new(self.context.clone())
            .get_all_imme_delivery(request)
            .await
    }

    pub async fn pre_add_order(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        DeliveryApi::new(self.context.clone())
            .pre_add_order(request)
            .await
    }

    pub async fn pre_cancel_order(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        DeliveryApi::new(self.context.clone())
            .pre_cancel_order(request)
            .await
    }

    pub async fn add_local_order(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        DeliveryApi::new(self.context.clone())
            .add_local_order(request)
            .await
    }

    pub async fn cancel_local_order(
        &self,
        request: &DeliveryRequest,
    ) -> Result<DeliveryResponse, WechatError> {
        DeliveryApi::new(self.context.clone())
            .cancel_local_order(request)
            .await
    }

    pub async fn bind_account(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        LogisticsApi::new(self.context.clone())
            .bind_account(request)
            .await
    }

    pub async fn get_all_account(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        LogisticsApi::new(self.context.clone())
            .get_all_account(request)
            .await
    }

    pub async fn get_all_delivery(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        LogisticsApi::new(self.context.clone())
            .get_all_delivery(request)
            .await
    }

    pub async fn get_order(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        LogisticsApi::new(self.context.clone())
            .get_order(request)
            .await
    }

    pub async fn add_order(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        LogisticsApi::new(self.context.clone())
            .add_order(request)
            .await
    }

    pub async fn get_path(
        &self,
        request: &LogisticsRequest,
    ) -> Result<LogisticsResponse, WechatError> {
        LogisticsApi::new(self.context.clone())
            .get_path(request)
            .await
    }

    pub async fn invoke_service(
        &self,
        request: &InvokeServiceRequest,
    ) -> Result<ServiceMarketResponse, WechatError> {
        ServiceMarketApi::new(self.context.clone())
            .invoke_service(request)
            .await
    }

    pub async fn verify_signature(
        &self,
        request: &VerifySignatureRequest,
    ) -> Result<VerifySignatureResponse, WechatError> {
        SoterApi::new(self.context.clone())
            .verify_signature(request)
            .await
    }

    pub async fn get_verify_id(
        &self,
        request: &GetVerifyIdRequest,
    ) -> Result<FaceResponse, WechatError> {
        FaceApi::new(self.context.clone())
            .get_verify_id(request)
            .await
    }

    pub async fn query_verify_info(
        &self,
        request: &QueryVerifyInfoRequest,
    ) -> Result<FaceResponse, WechatError> {
        FaceApi::new(self.context.clone())
            .query_verify_info(request)
            .await
    }

    pub async fn submit_pages(
        &self,
        request: &SubmitPagesRequest,
    ) -> Result<SubmitPagesResponse, WechatError> {
        WxsearchApi::new(self.context.clone())
            .submit_pages(request)
            .await
    }

    pub async fn add_user_action(
        &self,
        request: &AdvertisingRequest,
    ) -> Result<AdvertisingResponse, WechatError> {
        AdvertisingApi::new(self.context.clone())
            .add_user_action(request)
            .await
    }

    pub async fn add_user_action_set(
        &self,
        request: &AdvertisingRequest,
    ) -> Result<AdvertisingResponse, WechatError> {
        AdvertisingApi::new(self.context.clone())
            .add_user_action_set(request)
            .await
    }

    pub async fn get_user_action_set_reports(
        &self,
        request: &AdvertisingRequest,
    ) -> Result<AdvertisingResponse, WechatError> {
        AdvertisingApi::new(self.context.clone())
            .get_user_action_set_reports(request)
            .await
    }

    pub async fn get_user_action_sets(
        &self,
        request: &AdvertisingRequest,
    ) -> Result<AdvertisingResponse, WechatError> {
        AdvertisingApi::new(self.context.clone())
            .get_user_action_sets(request)
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
