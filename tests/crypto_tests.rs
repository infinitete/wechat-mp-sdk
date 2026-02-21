use aes::cipher::KeyIvInit;
use aes::Aes128;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use cbc::cipher::block_padding::Pkcs7;
use cbc::Encryptor;
use wechat_mp_sdk::crypto::{decrypt_user_data, verify_watermark, DecryptedUserData, Watermark};

type Aes128CbcEnc = Encryptor<Aes128>;

fn encrypt_aes_128_cbc(key: &[u8; 16], iv: &[u8; 16], plaintext: &str) -> Vec<u8> {
    use aes::cipher::BlockEncryptMut;

    let cipher = Aes128CbcEnc::new(key.into(), iv.into());
    let pt_bytes = plaintext.as_bytes();
    let pt_len = pt_bytes.len();

    // Allocate buffer with space for PKCS7 padding (always adds 1–16 bytes)
    let pad_len = 16 - (pt_len % 16);
    let mut buffer = vec![0u8; pt_len + pad_len];
    buffer[..pt_len].copy_from_slice(pt_bytes);

    // Let the library handle all PKCS7 padding — no manual padding
    let encrypted = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, pt_len)
        .unwrap();
    encrypted.to_vec()
}

#[test]
fn test_invalid_base64_encrypted_data() {
    let result = decrypt_user_data(
        "MTIzNDU2Nzg5MDEyMzQ1Ng==",
        "not-valid!!!",
        "MTIzNDU2Nzg5MDEyMzQ1",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("Invalid encrypted_data"));
}

#[test]
fn test_invalid_base64_iv() {
    let result = decrypt_user_data(
        "MTIzNDU2Nzg5MDEyMzQ1Ng==",
        "MTIzNDU2Nzg5MDEyMzQ1",
        "not-valid!!!",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("Invalid iv"));
}

#[test]
fn test_invalid_key_length() {
    let result = decrypt_user_data(
        "MTIzNDU2Nzg5",
        "MTIzNDU2Nzg5MDEyMzQ1Ng==",
        "MTIzNDU2Nzg5MDEyMzQ1",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("Invalid key length"));
}

#[test]
fn test_invalid_iv_length() {
    let result = decrypt_user_data(
        "MTIzNDU2Nzg5MDEyMzQ1Ng==",
        "MTIzNDU2Nzg5MDEyMzQ1Ng==",
        "MTIzNDU2",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("Invalid IV length"));
}

#[test]
fn test_key_length_16_is_valid() {
    let key = "MTIzNDU2Nzg5MDEyMzQ1Ng==";
    let result = decrypt_user_data(key, "MTIzNDU2Nzg5MDEyMzQ1Ng==", "MTIzNDU2Nzg5MDEyMzQ1");
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(err_msg.contains("Invalid IV length") || err_msg.contains("Decryption failed"));
}

#[test]
fn test_empty_session_key() {
    let result = decrypt_user_data("", "data", "MTIzNDU2Nzg5MDEyMzQ1");
    assert!(result.is_err());
}

#[test]
fn test_empty_encrypted_data() {
    let result = decrypt_user_data("MTIzNDU2Nzg5MDEyMzQ1Ng==", "", "MTIzNDU2Nzg5MDEyMzQ1");
    assert!(result.is_err());
}

#[test]
fn test_verify_watermark_success() {
    let data = DecryptedUserData::new(
        serde_json::json!({"openid": "test"}),
        Watermark::new(1234567890, "wx1234567890"),
    );
    let result = verify_watermark(&data, "wx1234567890");
    assert!(result.is_ok());
}

#[test]
fn test_verify_watermark_mismatch() {
    let data = DecryptedUserData::new(
        serde_json::json!({"openid": "test"}),
        Watermark::new(1234567890, "wx1234567890"),
    );
    let result = verify_watermark(&data, "wx9999999999");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("Watermark appid mismatch"));
}

#[test]
fn test_watermark_struct() {
    let watermark = Watermark::new(1234567890, "wxabcdef123456789");
    assert_eq!(watermark.timestamp(), 1234567890);
    assert_eq!(watermark.appid(), "wxabcdef123456789");
}

#[test]
fn test_decrypted_user_data_struct() {
    let data = DecryptedUserData::new(
        serde_json::json!({"nickName": "Test", "openid": "o123"}),
        Watermark::new(1234567890, "wx123"),
    );
    assert_eq!(data.data["nickName"], "Test");
    assert_eq!(data.watermark.appid(), "wx123");
}

#[test]
fn test_end_to_end_encryption_decryption_simple() {
    let key = b"simplekey1234567";
    let iv = b"simpleiv12345678";
    let plaintext =
        r#"{"openid":"test_user","watermark":{"timestamp":1609459200,"appid":"wxsimple"}}"#;

    let encrypted = encrypt_aes_128_cbc(key, iv, plaintext);

    let key_b64 = BASE64.encode(key);
    let iv_b64 = BASE64.encode(iv);
    let encrypted_b64 = BASE64.encode(&encrypted);

    let result = decrypt_user_data(&key_b64, &encrypted_b64, &iv_b64);

    assert!(result.is_ok(), "Decryption failed: {:?}", result.err());
    let decrypted = result.unwrap();
    assert_eq!(decrypted.data["openid"], "test_user");
}

#[test]
fn test_end_to_end_encryption_decryption_with_special_chars() {
    let key = b"key123456789012a";
    let iv = b"iv12345678901234";
    let plaintext = r#"{"nickName":"用户昵称","gender":1,"watermark":{"timestamp":1609459200,"appid":"wxaabbccddeefff"}}"#;

    let encrypted = encrypt_aes_128_cbc(key, iv, plaintext);

    let key_b64 = BASE64.encode(key);
    let iv_b64 = BASE64.encode(iv);
    let encrypted_b64 = BASE64.encode(&encrypted);

    let result = decrypt_user_data(&key_b64, &encrypted_b64, &iv_b64);

    assert!(result.is_ok(), "Decryption failed: {:?}", result.err());
    let decrypted = result.unwrap();
    assert_eq!(decrypted.data["nickName"], "用户昵称");
    assert_eq!(decrypted.data["gender"], 1);
}

#[test]
fn test_end_to_end_encryption_decryption_phone_number() {
    let key = b"sessionkey123456";
    let iv = b"initialvec123456";
    let plaintext = r#"{"phoneNumber":"13800138000","purePhoneNumber":"13800138000","countryCode":"86","watermark":{"timestamp":1612137600,"appid":"wxphonedemo123"}}"#;

    let encrypted = encrypt_aes_128_cbc(key, iv, plaintext);

    let key_b64 = BASE64.encode(key);
    let iv_b64 = BASE64.encode(iv);
    let encrypted_b64 = BASE64.encode(&encrypted);

    let result = decrypt_user_data(&key_b64, &encrypted_b64, &iv_b64);

    assert!(result.is_ok(), "Decryption failed: {:?}", result.err());
    let decrypted = result.unwrap();
    assert_eq!(decrypted.data["phoneNumber"], "13800138000");
    assert_eq!(decrypted.data["countryCode"], "86");
}

#[test]
fn test_end_to_end_multiple_fields() {
    let key = b"complexkey123456";
    let iv = b"complexiv1234567";
    let plaintext = r#"{"openId":"oXXXX_user_openid","unionId":"uXXXX_user_unionid","nickName":"John Doe","gender":1,"country":"China","province":"Guangdong","city":"Shenzhen","watermark":{"timestamp":1622505600,"appid":"wxmulti123456"}}"#;

    let encrypted = encrypt_aes_128_cbc(key, iv, plaintext);

    let key_b64 = BASE64.encode(key);
    let iv_b64 = BASE64.encode(iv);
    let encrypted_b64 = BASE64.encode(&encrypted);

    let result = decrypt_user_data(&key_b64, &encrypted_b64, &iv_b64);

    assert!(result.is_ok(), "Decryption failed: {:?}", result.err());
    let decrypted = result.unwrap();
    assert_eq!(decrypted.data["openId"], "oXXXX_user_openid");
    assert_eq!(decrypted.data["unionId"], "uXXXX_user_unionid");
    assert_eq!(decrypted.data["nickName"], "John Doe");
    assert_eq!(decrypted.data["gender"], 1);
    assert_eq!(decrypted.data["country"], "China");
    assert_eq!(decrypted.data["province"], "Guangdong");
    assert_eq!(decrypted.data["city"], "Shenzhen");
}

#[test]
fn test_end_to_end_boundary_exact_block() {
    let key = b"blockboundarykey";
    let iv = b"blockboundaryiv!";
    let plaintext = r#"{"openid":"block_boundary_test","watermark":{"timestamp":1609459200,"appid":"wxblocktest12345"}}"#;
    assert_eq!(
        plaintext.len() % 16,
        0,
        "Plaintext must be block-aligned for this test"
    );

    let encrypted = encrypt_aes_128_cbc(key, iv, plaintext);

    let key_b64 = BASE64.encode(key);
    let iv_b64 = BASE64.encode(iv);
    let encrypted_b64 = BASE64.encode(&encrypted);

    let result = decrypt_user_data(&key_b64, &encrypted_b64, &iv_b64);

    assert!(result.is_ok(), "Decryption failed: {:?}", result.err());
    let decrypted = result.unwrap();
    assert_eq!(decrypted.data["openid"], "block_boundary_test");
}

#[test]
fn test_end_to_end_boundary_two_blocks() {
    let key = b"twoblocktestkey1";
    let iv = b"twoblocktestivv1";
    let plaintext = r#"{"data":"This is a much longer string that exceeds one AES block size of 16 bytes for testing","watermark":{"timestamp":9999999999,"appid":"wxboundarytest"}}"#;

    let encrypted = encrypt_aes_128_cbc(key, iv, plaintext);

    let key_b64 = BASE64.encode(key);
    let iv_b64 = BASE64.encode(iv);
    let encrypted_b64 = BASE64.encode(&encrypted);

    let result = decrypt_user_data(&key_b64, &encrypted_b64, &iv_b64);

    assert!(result.is_ok(), "Decryption failed: {:?}", result.err());
    let decrypted = result.unwrap();
    assert_eq!(
        decrypted.data["data"],
        "This is a much longer string that exceeds one AES block size of 16 bytes for testing"
    );
}

#[test]
fn test_end_to_end_with_empty_fields() {
    let key: &[u8; 16] = b"1234567890123456";
    let iv: &[u8; 16] = b"abcdefghijklmnop";
    let plaintext = r#"{"openId":"","nickName":"","gender":0,"watermark":{"timestamp":1000000000,"appid":"wxempty"}}"#;

    let encrypted = encrypt_aes_128_cbc(key, iv, plaintext);

    let key_b64 = BASE64.encode(key);
    let iv_b64 = BASE64.encode(iv);
    let encrypted_b64 = BASE64.encode(&encrypted);

    let result = decrypt_user_data(&key_b64, &encrypted_b64, &iv_b64);

    assert!(result.is_ok(), "Decryption failed: {:?}", result.err());
    let decrypted = result.unwrap();
    assert_eq!(decrypted.data["openId"], "");
    assert_eq!(decrypted.data["nickName"], "");
}
