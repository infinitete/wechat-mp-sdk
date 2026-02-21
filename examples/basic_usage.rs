//! Basic usage example for wechat-mp-sdk
//!
//! Run with: cargo run --example basic_usage

use wechat_mp_sdk::{
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

    println!("Client created successfully!");
    println!("AppID: {}", client.appid());
    println!("Base URL: {}", client.base_url());

    Ok(())
}
