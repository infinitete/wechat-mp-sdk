//! WeChat Mini Program API modules
//!
//! This module contains submodules for various WeChat Mini Program APIs:
//!
//! - [`auth`] - Login authentication (code2Session)
//! - [`user`] - User information and phone number
//! - [`message`] - Customer service and subscribe messages
//! - [`qrcode`] - Mini Program codes, QR codes, and URL links
//!
//! ## Usage
//!
//! ```rust,ignore
//! use wechat_mp_sdk::api::auth::AuthApi;
//! use wechat_mp_sdk::api::user::UserApi;
//! use wechat_mp_sdk::token::TokenManager;
//! ```

pub mod auth;
pub mod message;
pub mod qrcode;
pub mod user;
