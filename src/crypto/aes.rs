//! AES-128-CBC decryption for WeChat encrypted user data

use aes::cipher::{BlockDecryptMut, KeyIvInit};
use aes::Aes128;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use cbc::cipher::block_padding::Pkcs7;
use cbc::Decryptor;

use crate::error::WechatError;
use crate::types::Watermark;

type Aes128CbcDecryptor = Decryptor<Aes128>;

/// Decrypted user data with watermark
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DecryptedUserData {
    /// Sensitive user data fields vary by scenario
    /// Common fields include: openId, unionId, nickName, gender, etc.
    #[serde(flatten)]
    pub data: serde_json::Value,
    /// Watermark for verification
    pub watermark: Watermark,
}

/// Decrypt WeChat encrypted user data.
///
/// WeChat encrypts sensitive user data using AES-128-CBC with:
/// - Key: session_key (base64 decoded, 16 bytes)
/// - IV: First 16 bytes of encrypted data (base64 decoded)
/// - Data: Rest of encrypted data (base64 decoded)
///
/// # Arguments
/// * `session_key` - Base64 encoded session key from login
/// * `encrypted_data` - Base64 encoded encrypted data from client
/// * `iv` - Base64 encoded IV from client
///
/// # Returns
/// Decrypted user data containing the user's information.
///
/// # Errors
/// Returns [`WechatError::Crypto`] if:
/// - Invalid base64 encoding
/// - Invalid key or IV length
/// - Decryption fails
/// - Parsed data is not valid JSON
pub fn decrypt_user_data(
    session_key: &str,
    encrypted_data: &str,
    iv: &str,
) -> Result<DecryptedUserData, WechatError> {
    // Decode base64
    let key = BASE64
        .decode(session_key)
        .map_err(|e| WechatError::Crypto(format!("Invalid session_key: {}", e)))?;

    let encrypted = BASE64
        .decode(encrypted_data)
        .map_err(|e| WechatError::Crypto(format!("Invalid encrypted_data: {}", e)))?;

    let iv_bytes = BASE64
        .decode(iv)
        .map_err(|e| WechatError::Crypto(format!("Invalid iv: {}", e)))?;

    // Validate key length (16 bytes for AES-128)
    if key.len() != 16 {
        return Err(WechatError::Crypto(format!(
            "Invalid key length: expected 16, got {}",
            key.len()
        )));
    }

    // Validate IV length (16 bytes for AES-CBC)
    if iv_bytes.len() != 16 {
        return Err(WechatError::Crypto(format!(
            "Invalid IV length: expected 16, got {}",
            iv_bytes.len()
        )));
    }

    // Create decryptor
    let decryptor = Aes128CbcDecryptor::new(key.as_slice().into(), iv_bytes.as_slice().into());

    // Decrypt with PKCS7 padding
    let mut buffer = encrypted;
    let decrypted = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| WechatError::Crypto(format!("Decryption failed: {:?}", e)))?;

    // Parse as JSON
    let json_str = std::str::from_utf8(decrypted)
        .map_err(|e| WechatError::Crypto(format!("Invalid UTF-8: {}", e)))?;

    let user_data: DecryptedUserData = serde_json::from_str(json_str)
        .map_err(|e| WechatError::Crypto(format!("Invalid JSON: {}", e)))?;

    Ok(user_data)
}

/// Verify watermark appid matches expected appid.
///
/// This should be called after decryption to ensure the data came from
/// the correct Mini Program (prevents attacks using data from other apps).
///
/// # Arguments
/// * `data` - Decrypted user data containing watermark
/// * `expected_appid` - Your Mini Program's appid
///
/// # Errors
/// Returns [`WechatError::Signature`] if the watermark appid does not match.
pub fn verify_watermark(data: &DecryptedUserData, expected_appid: &str) -> Result<(), WechatError> {
    if data.watermark.appid != expected_appid {
        return Err(WechatError::Signature(format!(
            "Watermark appid mismatch: expected {}, got {}",
            expected_appid, data.watermark.appid
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test vectors would require actual WeChat test data
    // These are placeholder tests

    #[test]
    fn test_invalid_base64_session_key() {
        let result = decrypt_user_data("not-valid-base64!!!", "data", "iv");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_base64_encrypted_data() {
        let result = decrypt_user_data("MTIzNDU2Nzg5MDEyMzQ1Ng==", "not-valid!!!", "iv");
        assert!(result.is_err());
    }
}
