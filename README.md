# wechat-mp-sdk

[![Crates.io](https://img.shields.io/crates/v/wechat-mp-sdk.svg)](https://crates.io/crates/wechat-mp-sdk)
[![Documentation](https://docs.rs/wechat-mp-sdk/badge.svg)](https://docs.rs/wechat-mp-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

微信小程序服务端 SDK for Rust。

## 功能特性

- 登录认证 (code2Session)
- 用户信息获取与解密
- 手机号获取
- Access Token 自动管理
- 小程序码/二维码生成
- URL Scheme/Link 生成
- 短链接生成
- 客服消息发送
- 订阅消息发送
- 模板消息管理

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
wechat-mp-sdk = "0.1"
```

### 可选依赖

如需启用额外的 HTTP Client 功能：

```toml
wechat-mp-sdk = { version = "0.1", features = ["native-tls"] }
```

## 快速开始

```rust
use wechat_mp_sdk::{
    WechatClient, WechatClientBuilder,
    api::auth::AuthApi,
    token::TokenManager,
    types::{AppId, AppSecret},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let appid = AppId::new("your_appid")?;
    let secret = AppSecret::new("your_secret")?;
    
    let client = WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .build()?;
    
    // 登录凭证校验
    let auth_api = AuthApi::new(client.clone());
    let login_response = auth_api.login("code_from_wx_login").await?;
    println!("OpenID: {}", login_response.openid);
    println!("Session Key: {}", login_response.session_key);
    
    Ok(())
}
```

## API 概览

### 客户端

```rust
use wechat_mp_sdk::{WechatClient, WechatClientBuilder, types::{AppId, AppSecret}};

// 创建客户端
let client = WechatClient::builder()
    .appid(AppId::new("your_appid")?)
    .secret(AppSecret::new("your_secret")?)
    .build()?;

// 自定义配置
let client = WechatClient::builder()
    .appid(AppId::new("your_appid")?)
    .secret(AppSecret::new("your_secret")?)
    .base_url("https://api.weixin.qq.com")  // 默认值
    .timeout(std::time::Duration::from_secs(30))  // 默认 30 秒
    .connect_timeout(std::time::Duration::from_secs(10))  // 默认 10 秒
    .build()?;
```

### 登录认证

```rust
use wechat_mp_sdk::api::auth::{AuthApi, LoginResponse};

let auth_api = AuthApi::new(client);

// 使用 wx.login() 获取的 code 进行登录
let response: LoginResponse = auth_api.login("code_from_miniprogram").await?;
println!("OpenID: {}", response.openid);
println!("Session Key: {}", response.session_key);
println!("UnionID: {:?}", response.unionid);
```

### 用户信息

```rust
use wechat_mp_sdk::api::user::UserApi;
use wechat_mp_sdk::token::TokenManager;

let user_api = UserApi::new(client.clone());
let token_manager = TokenManager::new(client);

// 获取用户手机号
let phone_response = user_api
    .get_phone_number(&token_manager, "code_from_getPhoneNumber")
    .await?;
println!("Phone: {}", phone_response.phone_info.phone_number);
```

### 客服消息

```rust
use wechat_mp_sdk::api::message::{
    MessageApi, Message, TextMessage, 
    MediaMessage, LinkMessage, MiniprogramPageMessage,
    SubscribeMessageData, SubscribeMessageValue
};
use wechat_mp_sdk::token::TokenManager;

let message_api = MessageApi::new(client.clone());
let token_manager = TokenManager::new(client);

// 发送文本消息
let message = Message::Text { 
    text: TextMessage::new("您好！") 
};
message_api
    .send_customer_service_message(&token_manager, "user_openid", message)
    .await?;

// 发送图片消息
let message = Message::Image { 
    image: MediaMessage::new("media_id_from_upload") 
};
message_api
    .send_customer_service_message(&token_manager, "user_openid", message)
    .await?;

// 发送订阅消息
let mut data = SubscribeMessageData::new();
data.insert("keyword1".to_string(), SubscribeMessageValue::new("内容"));
message_api
    .send_subscribe_message(
        &token_manager,
        "user_openid",
        "template_id",
        data,
        Some("pages/index/index"),
    )
    .await?;
```

### 小程序码

```rust
use wechat_mp_sdk::api::qrcode::{
    QrcodeApi, QrcodeOptions, UnlimitQrcodeOptions,
    UrlSchemeOptions, UrlSchemeExpire, UrlLinkOptions,
    ShortLinkOptions, LineColor
};
use wechat_mp_sdk::token::TokenManager;

let qrcode_api = QrcodeApi::new(client.clone());
let token_manager = TokenManager::new(client);

// 获取小程序码
let options = QrcodeOptions {
    path: Some("/pages/index/index".to_string()),
    width: Some(430),
    auto_color: Some(false),
    line_color: Some(LineColor { r: 0, g: 0, b: 0 }),
    is_hyaline: Some(false),
};
let bytes = qrcode_api.get_wxa_code(&token_manager, options).await?;
// bytes 是图片的二进制数据，可以保存为文件

// 获取不限定的小程序码
let options = UnlimitQrcodeOptions {
    scene: "abc".to_string(),
    page: Some("/pages/index/index".to_string()),
    width: Some(430),
    auto_color: None,
    line_color: None,
    is_hyaline: None,
};
let bytes = qrcode_api.get_wxa_code_unlimit(&token_manager, options).await?;

// 生成 URL Scheme
let options = UrlSchemeOptions {
    path: Some("/pages/index/index".to_string()),
    query: Some("id=123".to_string()),
    expire: Some(UrlSchemeExpire {
        expire_type: 1,
        expire_time: Some(1672531200),  // 过期时间戳
        expire_interval: None,
    }),
};
let scheme_url = qrcode_api.generate_url_scheme(&token_manager, options).await?;

// 生成 URL Link
let options = UrlLinkOptions {
    path: Some("/pages/index/index".to_string()),
    query: Some("id=123".to_string()),
    expire_type: Some(1),
    expire_time: Some(1672531200),
    expire_interval: None,
};
let link_url = qrcode_api.generate_url_link(&token_manager, options).await?;

// 生成短链接
let options = ShortLinkOptions {
    page_url: "https://example.com/page".to_string(),
};
let short_link = qrcode_api.generate_short_link(&token_manager, options).await?;
```

### Access Token 管理

```rust
use wechat_mp_sdk::token::TokenManager;

let token_manager = TokenManager::new(client);

// 获取 Token（自动缓存和刷新）
let token = token_manager.get_token().await?;
println!("Access Token: {}", token);

// 手动失效 Token
token_manager.invalidate().await;
```

### 数据解密

```rust
use wechat_mp_sdk::crypto;

let session_key = "session_key_from_login";
let encrypted_data = "encrypted_data_from_miniprogram";
let iv = "iv_from_miniprogram";

let decrypted = crypto::decrypt_user_data(session_key, encrypted_data, iv)?;
let user_info: UserInfo = serde_json::from_str(&decrypted)?;
```

## 错误处理

```rust
use wechat_mp_sdk::WechatError;

match result {
    Ok(response) => { /* 处理成功响应 */ }
    Err(WechatError::Api { code, message }) => {
        eprintln!("API 错误: {} - {}", code, message);
    }
    Err(WechatError::Config(msg)) => {
        eprintln!("配置错误: {}", msg);
    }
    Err(WechatError::Http(e)) => {
        eprintln!("网络错误: {}", e);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

## 文档

查看 [docs.rs/wechat-mp-sdk](https://docs.rs/wechat-mp-sdk) 获取完整 API 文档。

运行以下命令生成本地文档：

```bash
cargo doc --open
```

## 示例

更多示例请查看 [examples/](examples/) 目录。

## 许可证

MIT License
