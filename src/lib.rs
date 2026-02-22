//! WeChat Mini Program SDK for Rust
//!
//! A complete Rust SDK for the WeChat Mini Program server-side APIs,
//! covering **128 endpoints** across 24 categories.
//!
//! ## API Coverage
//!
//! | Category | Endpoints |
//! |----------|-----------|
//! | Login / Session | 3 |
//! | Access Token | 2 |
//! | OpenAPI Management | 8 |
//! | Security | 3 |
//! | User Info | 5 |
//! | QR Code / Links | 9 |
//! | Customer Service | 4 |
//! | Subscribe Messages | 10 |
//! | Analytics | 11 |
//! | Operations | 10 |
//! | Image / OCR | 8 |
//! | Plugin | 2 |
//! | Nearby Mini Programs | 4 |
//! | Cloud Development | 10 |
//! | Live Streaming | 9 |
//! | Hardware / IoT | 6 |
//! | Instant Delivery | 5 |
//! | Logistics | 6 |
//! | Service Market | 1 |
//! | Biometric Auth | 1 |
//! | Face Verification | 2 |
//! | WeChat Search | 1 |
//! | Advertising | 4 |
//! | WeChat KF | 3 |
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use wechat_mp_sdk::{WechatMp, types::{AppId, AppSecret}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let wechat = WechatMp::builder()
//!         .appid(AppId::new("wx1234567890abcdef")?)
//!         .secret(AppSecret::new("your_secret")?)
//!         .build()?;
//!
//!     // Login with code from wx.login()
//!     let session = wechat.auth_login("code_from_miniprogram").await?;
//!     println!("OpenID: {}", session.openid);
//!
//!     // Get phone number
//!     let phone = wechat.get_phone_number("code_from_getPhoneNumber").await?;
//!     println!("Phone: {}", phone.phone_info.phone_number);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - [`api`] - WeChat API modules (auth, user, message, qrcode, analytics, etc.)
//! - [`client`] - HTTP client for API calls
//! - [`crypto`] - Data decryption utilities
//! - [`error`] - Error types
//! - [`token`] - Access token management (internal, for advanced users)
//! - [`types`] - Type definitions for WeChat API entities
//!
//! ## Error Handling
//!
//! The SDK uses the [`WechatError`] enum for error handling:
//!
//! ```rust,ignore
//! use wechat_mp_sdk::WechatError;
//!
//! match result {
//!     Ok(response) => { /* handle success */ }
//!     Err(WechatError::Api { code, message }) => {
//!         eprintln!("API error: {} - {}", code, message);
//!     }
//!     Err(WechatError::Http(e)) => {
//!         eprintln!("HTTP error: {}", e);
//!     }
//!     Err(e) => {
//!         eprintln!("Other error: {}", e);
//!     }
//! }
//! ```

pub mod api;
pub mod client;
pub mod crypto;
pub mod error;
pub mod middleware;
pub mod token;
pub mod types;
mod utils;

pub use client::{WechatClient, WechatClientBuilder, WechatMp, WechatMpBuilder};
pub use error::WechatError;
