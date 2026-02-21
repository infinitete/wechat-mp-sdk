//! Message sending example
//!
//! This example demonstrates sending customer service
//! and subscribe messages.
//!
//! Run with: cargo run --example message_send

use std::collections::HashMap;

use wechat_mp_sdk::{
    api::message::{Message, MessageApi, SubscribeMessageValue, TextMessage},
    token::TokenManager,
    types::{AppId, AppSecret},
    WechatClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let appid = AppId::new("wx1234567890abcdef")?;
    let secret = AppSecret::new("your_app_secret_here")?;

    let client = WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .build()?;

    let token_manager = TokenManager::new(client.clone());
    let message_api = MessageApi::new(client);

    let text_message = Message::Text {
        text: TextMessage::new("Hello from Rust SDK!"),
    };

    match message_api
        .send_customer_service_message(&token_manager, "user_openid", text_message)
        .await
    {
        Ok(_) => println!("Customer service message sent!"),
        Err(e) => eprintln!("Failed to send: {}", e),
    }

    let mut data = HashMap::new();
    data.insert(
        "thing1".to_string(),
        SubscribeMessageValue {
            value: "测试数据".to_string(),
        },
    );

    match message_api
        .send_subscribe_message(
            &token_manager,
            "user_openid",
            "template_id",
            data,
            Some("pages/index"),
        )
        .await
    {
        Ok(_) => println!("Subscribe message sent!"),
        Err(e) => eprintln!("Failed to send: {}", e),
    }

    Ok(())
}
