use wechat_mp_sdk::api::auth::LoginResponse;
use wechat_mp_sdk::api::message::{
    MediaMessage, MediaType, Message, MiniprogramPageMessage, TextMessage,
};
use wechat_mp_sdk::api::qrcode::{LineColor, QrcodeOptions, UnlimitQrcodeOptions};
use wechat_mp_sdk::api::user::{PhoneInfo, UserInfo, Watermark};

#[test]
fn test_login_response_parsing_success() {
    let json =
        r#"{"openid":"oXXX","session_key":"abc==","unionid":"oYYY","errcode":0,"errmsg":""}"#;
    let response: LoginResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.openid, "oXXX");
    assert_eq!(response.session_key, "abc==");
    assert_eq!(response.unionid, Some("oYYY".to_string()));
    assert!(response.is_success());
}

#[test]
fn test_login_response_parsing_without_unionid() {
    let json = r#"{"openid":"oXXX","session_key":"abc==","errcode":0,"errmsg":""}"#;
    let response: LoginResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.openid, "oXXX");
    assert_eq!(response.session_key, "abc==");
    assert!(response.unionid.is_none());
    assert!(response.is_success());
}

#[test]
fn test_login_response_error() {
    let json = r#"{"openid":"","session_key":"","errcode":40029,"errmsg":"invalid code"}"#;
    let response: LoginResponse = serde_json::from_str(json).unwrap();
    assert!(!response.is_success());
    assert_eq!(response.errcode, 40029);
}

#[test]
fn test_login_response_empty_openid() {
    let json = r#"{"openid":"","session_key":"","errcode":0,"errmsg":""}"#;
    let response: LoginResponse = serde_json::from_str(json).unwrap();
    assert!(response.is_success());
    assert!(response.openid.is_empty());
}

#[test]
fn test_user_info_parsing_full() {
    let json = r#"{
        "nick_name":"John",
        "avatar_url":"https://example.com/avatar.jpg",
        "gender":1,
        "city":"Beijing",
        "province":"Beijing",
        "country":"China",
        "language":"zh_CN"
    }"#;
    let user_info: UserInfo = serde_json::from_str(json).unwrap();
    assert_eq!(user_info.nick_name, Some("John".to_string()));
    assert_eq!(
        user_info.avatar_url,
        Some("https://example.com/avatar.jpg".to_string())
    );
    assert_eq!(user_info.gender, 1);
    assert_eq!(user_info.city, Some("Beijing".to_string()));
    assert_eq!(user_info.province, Some("Beijing".to_string()));
    assert_eq!(user_info.country, Some("China".to_string()));
    assert_eq!(user_info.language, Some("zh_CN".to_string()));
}

#[test]
fn test_user_info_parsing_minimal() {
    let json = r#"{}"#;
    let user_info: UserInfo = serde_json::from_str(json).unwrap();
    assert!(user_info.nick_name.is_none());
    assert!(user_info.avatar_url.is_none());
    assert_eq!(user_info.gender, 0);
    assert!(user_info.city.is_none());
}

#[test]
fn test_user_info_gender_values() {
    for gender in [0u8, 1, 2] {
        let json = format!(r#"{{"gender":{}}}"#, gender);
        let user_info: UserInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(user_info.gender, gender);
    }
}

#[test]
fn test_phone_info_parsing() {
    let json = r#"{
        "phone_number":"+8613800138000",
        "pure_phone_number":"13800138000",
        "country_code":"86",
        "watermark":{
            "timestamp":1234567890,
            "appid":"wx1234567890"
        }
    }"#;
    let phone_info: PhoneInfo = serde_json::from_str(json).unwrap();
    assert_eq!(phone_info.phone_number, "+8613800138000");
    assert_eq!(phone_info.pure_phone_number, "13800138000");
    assert_eq!(phone_info.country_code, "86");
    assert_eq!(phone_info.watermark.timestamp, 1234567890);
    assert_eq!(phone_info.watermark.appid, "wx1234567890");
}

#[test]
fn test_watermark_parsing() {
    let json = r#"{"timestamp":1234567890,"appid":"wx1234567890abcdef"}"#;
    let watermark: Watermark = serde_json::from_str(json).unwrap();
    assert_eq!(watermark.timestamp, 1234567890);
    assert_eq!(watermark.appid, "wx1234567890abcdef");
}

#[test]
fn test_qrcode_options_full() {
    let options = QrcodeOptions {
        path: Some("/pages/index".to_string()),
        width: Some(430),
        auto_color: Some(true),
        line_color: Some(LineColor { r: 0, g: 0, b: 0 }),
        is_hyaline: Some(true),
    };
    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("/pages/index"));
    assert!(json.contains("430"));
    assert!(json.contains("true"));
    assert!(json.contains("hyaline"));
}

#[test]
fn test_qrcode_options_minimal() {
    let options = QrcodeOptions {
        path: Some("/pages/index".to_string()),
        width: None,
        auto_color: None,
        line_color: None,
        is_hyaline: None,
    };
    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("/pages/index"));
    assert!(!json.contains("width"));
    assert!(!json.contains("auto_color"));
}

#[test]
fn test_qrcode_options_serialization_order() {
    let options = QrcodeOptions {
        path: Some("pages/index".to_string()),
        width: Some(300),
        auto_color: None,
        line_color: None,
        is_hyaline: None,
    };
    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("path"));
    assert!(json.contains("pages/index"));
}

#[test]
fn test_unlimit_qrcode_options() {
    let options = UnlimitQrcodeOptions {
        scene: "abc123".to_string(),
        page: Some("/pages/index".to_string()),
        width: Some(430),
        auto_color: None,
        line_color: None,
        is_hyaline: None,
    };
    assert_eq!(options.scene, "abc123");
    assert!(options.page.is_some());
}

#[test]
fn test_line_color() {
    let color = LineColor {
        r: 255,
        g: 128,
        b: 0,
    };
    let json = serde_json::to_string(&color).unwrap();
    assert!(json.contains("255"));
    assert!(json.contains("128"));
    assert!(json.contains("0"));
}

#[test]
fn test_text_message() {
    let msg = TextMessage::new("Hello world");
    assert_eq!(msg.content, "Hello world");
}

#[test]
fn test_text_message_serialization() {
    let msg = TextMessage::new("Test message");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("Test message"));
}

#[test]
fn test_media_message() {
    let msg = MediaMessage::new("media_id_12345");
    assert_eq!(msg.media_id, "media_id_12345");
}

#[test]
fn test_media_message_serialization() {
    let msg = MediaMessage::new("media_id_abc");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("media_id_abc"));
}

#[test]
fn test_miniprogram_page_message() {
    let msg =
        MiniprogramPageMessage::new("Title", "appid123", "pages/index/index", "thumb_media_id");
    assert_eq!(msg.title, "Title");
    assert_eq!(msg.appid, "appid123");
    assert_eq!(msg.pagepath, "pages/index/index");
    assert_eq!(msg.thumb_media_id, "thumb_media_id");
}

#[test]
fn test_message_text_variant() {
    let msg = Message::Text {
        text: TextMessage::new("Hello!"),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("text"));
    assert!(json.contains("Hello!"));
}

#[test]
fn test_message_image_variant() {
    let msg = Message::Image {
        image: MediaMessage::new("media_id_123"),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("image"));
    assert!(json.contains("media_id_123"));
}

#[test]
fn test_message_link_variant() {
    let msg = Message::Link {
        link: wechat_mp_sdk::api::message::LinkMessage::new(
            "Title",
            "Description",
            "https://example.com",
            "https://example.com/thumb.jpg",
        ),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("link"));
    assert!(json.contains("Title"));
}

#[test]
fn test_message_miniprogrampage_variant() {
    let msg = Message::Miniprogrampage {
        miniprogrampage: MiniprogramPageMessage::new(
            "Title",
            "appid123",
            "pages/index/index",
            "thumb_id",
        ),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("miniprogrampage"));
    assert!(json.contains("Title"));
}

#[test]
fn test_media_type_as_str() {
    assert_eq!(MediaType::Image.as_str(), "image");
    assert_eq!(MediaType::Voice.as_str(), "voice");
    assert_eq!(MediaType::Video.as_str(), "video");
    assert_eq!(MediaType::Thumb.as_str(), "thumb");
}

#[test]
fn test_qrcode_options_with_complex_line_color() {
    let options = QrcodeOptions {
        path: Some("/pages/test".to_string()),
        width: Some(500),
        auto_color: Some(false),
        line_color: Some(LineColor {
            r: 255,
            g: 0,
            b: 128,
        }),
        is_hyaline: Some(false),
    };
    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("255"));
    assert!(json.contains("0"));
    assert!(json.contains("128"));
}

#[test]
fn test_login_response_with_special_chars_in_errmsg() {
    let json = r#"{"openid":"oXXX","session_key":"abc==","errcode":40029,"errmsg":"invalid code, hints: [ req_id: ABC123 ]"}"#;
    let response: LoginResponse = serde_json::from_str(json).unwrap();
    assert!(!response.is_success());
    assert!(response.errmsg.contains("invalid code"));
}
