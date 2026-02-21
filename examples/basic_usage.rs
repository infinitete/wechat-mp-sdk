//! Basic usage example for wechat-mp-sdk
//!
//! Run with: cargo run --example basic_usage

use wechat_mp_sdk::{
    types::{AppId, AppSecret},
    WechatMp,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wechat = WechatMp::builder()
        .appid(AppId::new("wx1234567890abcdef")?)
        .secret(AppSecret::new("your_app_secret_here")?)
        .build()?;

    println!("Client created successfully!");
    println!("AppID: {}", wechat.appid());

    Ok(())
}
