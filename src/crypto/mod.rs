//! Cryptography utilities for WeChat Mini Program data decryption
//!
//! Provides utilities for decrypting user data received from WeChat Mini Program.
//!
//! ## Security
//!
//! The session key used for decryption should be handled securely and never
//! exposed to the client-side code.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use wechat_mp_sdk::crypto;
//!
//! let session_key = "session_key_from_login";
//! let encrypted_data = "encrypted_data_from_miniprogram";
//! let iv = "iv_from_miniprogram";
//!
//! let decrypted = crypto::decrypt_user_data(session_key, encrypted_data, iv)?;
//! let user_info: UserInfo = serde_json::from_str(&decrypted)?;
//! ```

pub mod aes;

pub use crate::types::Watermark;
pub use aes::{decrypt_user_data, verify_watermark, DecryptedUserData};
