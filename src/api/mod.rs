//! WeChat Mini Program API modules
//!
//! This module contains submodules for various WeChat Mini Program APIs:
//!
//! - [`common`] - Shared response primitives, pagination, and date range types
//! - [`auth`] - Login authentication (code2Session)
//! - [`customer_service`] - Customer service messages
//! - [`media`] - Temporary media upload and download
//! - [`subscribe`] - Subscribe messages and template management
//! - [`qrcode`] - Mini Program codes, QR codes, and URL links
//! - [`template`] - Template message management
//! - [`user`] - User information and phone number
//! - [`openapi`] - OpenAPI quota and diagnostic management
//! - [`wechat_kf`] - WeChat open customer service account binding
//! - [`security`] - Content security checks
//! - [`analytics`] - Data analytics and visit trends
//! - [`operations`] - Mini program operations and logs
//! - [`plugin`] - Plugin application management
//! - [`nearby`] - Nearby points of interest
//! - [`cloud`] - WeChat Cloud Base (TCB)
//! - [`live`] - Live streaming rooms and goods
//! - [`hardware`] - IoT hardware device messaging
//! - [`ocr`] - Image processing and OCR
//! - [`delivery`] - Instant/local delivery orders
//! - [`logistics`] - Express delivery and tracking
//! - [`service_market`] - WeChat service marketplace
//! - [`soter`] - SOTER biometric authentication
//! - [`face`] - Face identity verification
//! - [`wxsearch`] - WeChat search page submission
//! - [`advertising`] - Advertising user action tracking
//!
//! ## Usage
//!
//! ```rust,ignore
//! use wechat_mp_sdk::{WechatMp, types::{AppId, AppSecret}};
//! ```

pub mod advertising;
pub mod analytics;
pub mod auth;
pub mod cloud;
pub mod common;
pub mod customer_service;
pub mod delivery;
pub mod endpoint_inventory;
pub mod face;
pub mod hardware;
pub mod live;
pub mod logistics;
pub mod media;
pub mod nearby;
pub mod ocr;
pub mod openapi;
pub mod operations;
pub mod plugin;
pub mod qrcode;
pub mod security;
pub mod service_market;
pub mod soter;
pub mod subscribe;
pub mod template;
pub mod r#trait;
pub mod user;
pub mod wechat_kf;
pub mod wxsearch;

pub use advertising::{AdvertisingApi, AdvertisingRequest, AdvertisingResponse};
pub use analytics::{
    AnalyticsApi, AnalyticsDateRangeRequest, AnalyticsResponse, PerformanceDataRequest,
};
pub use cloud::{
    CloudApi, CloudDatabaseRequest, CloudResponse, DelayedFunctionTaskRequest,
    DeleteCloudFileRequest, DownloadFileLinkRequest, InvokeCloudFunctionRequest,
    SendCloudBaseSmsRequest, UploadFileLinkRequest,
};
pub use common::{
    ApiResponseBase, DateRangeRequest, PaginatedRequest, PaginatedResponse, WechatApiResponse,
};
pub use customer_service::{
    CustomerServiceApi, LinkMessage, MediaMessage, Message, MiniProgramPageMessage, TextMessage,
};
pub use delivery::{DeliveryApi, DeliveryRequest, DeliveryResponse};
pub use face::{FaceApi, FaceResponse, GetVerifyIdRequest, QueryVerifyInfoRequest};
pub use hardware::{HardwareApi, HardwareRequest, HardwareResponse};
pub use live::{DeleteRoomRequest, GetLiveInfoRequest, LiveApi, LiveRequest, LiveResponse};
pub use logistics::{LogisticsApi, LogisticsRequest, LogisticsResponse};
pub use media::{MediaApi, MediaType, MediaUploadResponse};
pub use nearby::{
    AddNearbyPoiRequest, DeleteNearbyPoiRequest, NearbyApi, NearbyPoiListRequest, NearbyResponse,
    NearbyShowStatusRequest,
};
pub use ocr::{IdCardOcrRequest, OcrApi, OcrImageRequest, OcrResponse};
pub use operations::{
    FeedbackMediaRequest, FeedbackRequest, JsErrDetailRequest, JsErrListRequest, OperationsApi,
    OperationsResponse, RealtimeLogSearchRequest,
};
pub use plugin::{ManagePluginApplicationRequest, ManagePluginRequest, PluginApi, PluginResponse};
pub use r#trait::{WechatApi, WechatContext};
pub use security::{
    MediaCheckAsyncResponse, MsgSecCheckDetail, MsgSecCheckResponse, MsgSecCheckResult,
    SecurityApi, UserRiskRankOptions, UserRiskRankResponse,
};
pub use service_market::{InvokeServiceRequest, ServiceMarketApi, ServiceMarketResponse};
pub use soter::{SoterApi, VerifySignatureRequest, VerifySignatureResponse};
pub use subscribe::{
    AddTemplateResponse, CategoryInfo, CategoryListResponse, GetUserNotifyRequest, Lang,
    MiniProgramState, PubTemplateKeywordInfo, PubTemplateKeywordResponse, PubTemplateTitleInfo,
    PubTemplateTitleListResponse, SubscribeApi, SubscribeMessageData, SubscribeMessageOptions,
    SubscribeMessageValue, TemplateInfo, TemplateListResponse, UserNotifyExtRequest,
    UserNotifyRequest, UserNotifyResponse,
};
pub use template::TemplateApi;
pub use wechat_kf::{KfWorkBoundResponse, KfWorkInfo, WechatKfApi};
pub use wxsearch::{SubmitPagesRequest, SubmitPagesResponse, WxsearchApi};
