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
//!
//! ## Usage
//!
//! ```rust,ignore
//! use wechat_mp_sdk::{WechatMp, types::{AppId, AppSecret}};
//! ```

pub mod auth;
pub mod common;
pub mod customer_service;
pub mod endpoint_inventory;
pub mod media;
pub mod qrcode;
pub mod subscribe;
pub mod template;
pub mod r#trait;
pub mod user;

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
