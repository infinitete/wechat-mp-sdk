# wechat-mp-sdk

[![Crates.io](https://img.shields.io/crates/v/wechat-mp-sdk.svg)](https://crates.io/crates/wechat-mp-sdk)
[![Documentation](https://docs.rs/wechat-mp-sdk/badge.svg)](https://docs.rs/wechat-mp-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

微信小程序服务端 SDK for Rust。

## 功能特性

- 登录认证 (code2Session)
- 用户信息获取与解密
- 手机号获取
- Access Token 自动管理（支持并发安全、单飞模式）
- 小程序码/二维码生成
- URL Scheme/Link 生成
- 短链接生成
- 客服消息发送
- 订阅消息发送
- 模板消息管理
- 临时素材上传/获取

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
    let appid = AppId::new("wx1234567890abcdef")?;  // AppID 必须以 wx 开头，18 字符
    let secret = AppSecret::new("your_secret".to_string())?;
    
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
    .appid(AppId::new("wx1234567890abcdef")?)
    .secret(AppSecret::new("your_secret".to_string())?)
    .build()?;

// 自定义配置
let client = WechatClient::builder()
    .appid(AppId::new("wx1234567890abcdef")?)
    .secret(AppSecret::new("your_secret".to_string())?)
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

### 临时素材上传

```rust
use wechat_mp_sdk::api::message::{MessageApi, MediaType};
use wechat_mp_sdk::token::TokenManager;

let message_api = MessageApi::new(client.clone());
let token_manager = TokenManager::new(client);

// 上传临时素材
let image_data = std::fs::read("image.png")?;
let response = message_api
    .upload_temp_media(&token_manager, MediaType::Image, &image_data, "image.png")
    .await?;
println!("Media ID: {}", response.media_id);
println!("Expires in: {}s", response.expires_in);
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
use std::time::Duration;

// 使用默认配置
let token_manager = TokenManager::new(client);

// 使用自定义配置
let token_manager = TokenManager::builder(client)
    .max_retries(5)                           // 最大重试次数（默认 3）
    .retry_delay_ms(200)                     // 重试延迟（默认 100ms）
    .refresh_buffer(Duration::from_secs(600)) // 刷新缓冲时间（默认 300s）
    .build();

// 获取 Token（自动缓存和刷新，并发安全）
let token = token_manager.get_token().await?;
println!("Access Token: {}", token);

// 手动失效 Token
token_manager.invalidate().await;
```

TokenManager 特性：
- **自动缓存**: Token 有效期内复用，避免重复请求
- **自动刷新**: Token 过期前自动刷新（默认提前 5 分钟）
- **并发安全**: 多个并发请求共享同一 Token
- **单飞模式**: 并发请求只触发一次 API 调用
- **智能重试**: 自动重试临时性错误（如系统繁忙 -1、频率限制 45009）

### 数据解密

```rust
use wechat_mp_sdk::crypto::{self, DecryptedUserData};

let session_key = "session_key_from_login";
let encrypted_data = "encrypted_data_from_miniprogram";
let iv = "iv_from_miniprogram";

// 解密用户数据
let decrypted: DecryptedUserData = crypto::decrypt_user_data(
    session_key,
    encrypted_data,
    iv,
)?;

// 校验 watermark
crypto::verify_watermark(&decrypted, "your_appid")?;

// 访问解密后的数据
if let Some(open_id) = &decrypted.open_id {
    println!("OpenID: {}", open_id);
}
```

## 错误处理

```rust
use wechat_mp_sdk::WechatError;

match result {
    Ok(response) => { /* 处理成功响应 */ }
    Err(WechatError::Api { code, message }) => {
        eprintln!("API 错误: {} - {}", code, message);
    }
    Err(WechatError::Token(msg)) => {
        eprintln!("Token 错误: {}", msg);
    }
    Err(WechatError::Config(msg)) => {
        eprintln!("配置错误: {}", msg);
    }
    Err(WechatError::Http(e)) => {
        eprintln!("网络错误: {}", e);
    }
    Err(WechatError::Crypto(msg)) => {
        eprintln!("加解密错误: {}", msg);
    }
    Err(WechatError::NotSupported(msg)) => {
        eprintln!("功能不支持: {}", msg);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

## 类型安全

SDK 使用强类型 ID 防止参数混用：

```rust
use wechat_mp_sdk::types::{AppId, OpenId, AppSecret};

// AppId 校验：必须以 wx 开头，18 字符
let appid = AppId::new("wx1234567890abcdef")?;

// OpenId 校验：20-40 字符
let openid = OpenId::new("o6_bmjrPTlm6_2sgVt7hMZOPfL2M")?;

// 类型安全：无法混用不同类型的 ID
fn send_message(to: OpenId) { /* ... */ }
// send_message(appid)  // 编译错误！
```

## 文档

查看 [docs.rs/wechat-mp-sdk](https://docs.rs/wechat-mp-sdk) 获取完整 API 文档。

运行以下命令生成本地文档：

```bash
cargo doc --open
```

## 示例

更多示例请查看 [examples/](examples/) 目录。

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_login

# 运行测试并显示输出
cargo test -- --nocapture
```

## 许可证

MIT License
