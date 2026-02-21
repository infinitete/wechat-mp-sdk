//! Message sending example
//!
//! This example demonstrates sending customer service
//! and subscribe messages.
//!
//! Run with: cargo run --example message_send

use std::collections::HashMap;

use wechat_mp_sdk::{
    api::{Message, SubscribeMessageOptions, SubscribeMessageValue, TextMessage},
    types::{AppId, AppSecret, OpenId},
    WechatMp,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wechat = WechatMp::builder()
        .appid(AppId::new("wx1234567890abcdef")?)
        .secret(AppSecret::new("your_app_secret_here")?)
        .build()?;

    // Send customer service text message
    let text_message = Message::Text {
        text: TextMessage::new("Hello from Rust SDK!"),
    };

    match wechat
        .send_customer_service_message("user_openid", text_message)
        .await
    {
        Ok(_) => println!("Customer service message sent!"),
        Err(e) => eprintln!("Failed to send: {}", e),
    }

    // Send subscribe message
    let mut data = HashMap::new();
    data.insert(
        "thing1".to_string(),
        SubscribeMessageValue {
            value: "测试数据".to_string(),
        },
    );

    let options = SubscribeMessageOptions {
        touser: OpenId::new("o_user_openid_12345678901").unwrap(),
        template_id: "template_id".to_string(),
        data,
        page: Some("pages/index".to_string()),
        miniprogram_state: None,
        lang: None,
    };

    match wechat.send_subscribe_message(options).await {
        Ok(_) => println!("Subscribe message sent!"),
        Err(e) => eprintln!("Failed to send: {}", e),
    }

    Ok(())
}
