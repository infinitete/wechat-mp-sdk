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

pub use common::{
    ApiResponseBase, DateRangeRequest, PaginatedRequest, PaginatedResponse, WechatApiResponse,
};
pub use customer_service::{
    CustomerServiceApi, LinkMessage, MediaMessage, Message, MiniProgramPageMessage, TextMessage,
};
pub use media::{MediaApi, MediaType, MediaUploadResponse};
pub use r#trait::{WechatApi, WechatContext};
pub use subscribe::{
    AddTemplateResponse, CategoryInfo, CategoryListResponse, Lang, MiniProgramState, SubscribeApi,
    SubscribeMessageData, SubscribeMessageOptions, SubscribeMessageValue, TemplateInfo,
    TemplateListResponse,
};
pub use template::TemplateApi;
