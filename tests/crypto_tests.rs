use wechat_mp_sdk::crypto::{decrypt_user_data, verify_watermark, DecryptedUserData, Watermark};

#[test]
fn test_invalid_base64_session_key() {
    let result = decrypt_user_data("not-valid-base64!!!", "data", "iv");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("Invalid session_key"));
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
    let data = DecryptedUserData {
        data: serde_json::json!({"openid": "test"}),
        watermark: Watermark {
            timestamp: 1234567890,
            appid: "wx1234567890".to_string(),
        },
    };
    let result = verify_watermark(&data, "wx1234567890");
    assert!(result.is_ok());
}

#[test]
fn test_verify_watermark_mismatch() {
    let data = DecryptedUserData {
        data: serde_json::json!({"openid": "test"}),
        watermark: Watermark {
            timestamp: 1234567890,
            appid: "wx1234567890".to_string(),
        },
    };
    let result = verify_watermark(&data, "wx9999999999");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("Watermark appid mismatch"));
}

#[test]
fn test_watermark_struct() {
    let watermark = Watermark {
        timestamp: 1234567890,
        appid: "wxabcdef123456789".to_string(),
    };
    assert_eq!(watermark.timestamp, 1234567890);
    assert_eq!(watermark.appid, "wxabcdef123456789");
}

#[test]
fn test_decrypted_user_data_struct() {
    let data = DecryptedUserData {
        data: serde_json::json!({"nickName": "Test", "openid": "o123"}),
        watermark: Watermark {
            timestamp: 1234567890,
            appid: "wx123".to_string(),
        },
    };
    assert_eq!(data.data["nickName"], "Test");
    assert_eq!(data.watermark.appid, "wx123");
}
