//! Complete login flow example
//!
//! This example demonstrates the full login flow:
//! 1. Receive code from mini program client
//! 2. Call code2Session to get openid and session_key
//! 3. Optionally decrypt user data
//!
//! Run with: cargo run --example login_flow

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

    let js_code = "code_from_wx_login";

    match wechat.auth_login(js_code).await {
        Ok(response) => {
            if response.is_success() {
                println!("Login successful!");
                println!("OpenID: {}", response.openid);
                if let Some(unionid) = response.unionid {
                    println!("UnionID: {}", unionid);
                }
                println!("Session Key: {}", response.session_key);
            } else {
                eprintln!("Login failed: {}", response.errmsg());
            }
        }
        Err(e) => {
            eprintln!("Login error: {}", e);
        }
    }

    Ok(())
}
