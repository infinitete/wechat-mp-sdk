//! WeChat Mini Program SDK for Rust
//!
//! A Rust SDK for interacting with the WeChat Mini Program APIs.
//!
//! ## Features
//!
//! - Login authentication via code2Session
//! - User information decryption
//! - Phone number retrieval
//! - Automatic access token management with caching
//! - Mini Program code and QR code generation
//! - URL Scheme/Link generation
//! - Short link generation
//! - Customer service message sending
//! - Subscribe message sending
//! - Template message management
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
//!     // Use the client with API modules...
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - [`api`] - WeChat API modules (auth, user, message, qrcode)
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

pub use client::{WechatClient, WechatClientBuilder, WechatMp, WechatMpBuilder};
pub use error::WechatError;
