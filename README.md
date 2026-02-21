# wechat-mp-sdk

[![Crates.io](https://img.shields.io/crates/v/wechat-mp-sdk.svg)](https://crates.io/crates/wechat-mp-sdk)
[![Documentation](https://docs.rs/wechat-mp-sdk/badge.svg)](https://docs.rs/wechat-mp-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

微信小程序服务端 SDK for Rust。

## 功能特性

- 登录认证 (code2Session)
- 用户信息获取与解密
- 手机号获取
- Access Token 自动管理（内置于客户端，支持并发安全、单飞模式）
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
wechat-mp-sdk = "0.2"
```

### 可选依赖

如需启用额外的 HTTP Client 功能：

```toml
wechat-mp-sdk = { version = "0.2", features = ["native-tls"] }
```

## 快速开始

```rust
use wechat_mp_sdk::{WechatMp, types::{AppId, AppSecret}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let wechat = WechatMp::builder()
        .appid(AppId::new("wx1234567890abcdef")?)  // AppID 必须以 wx 开头，18 字符
        .secret(AppSecret::new("your_secret")?)
        .build()?;
    
    // 登录凭证校验（无需手动管理 Token）
    let login_response = wechat.auth_login("code_from_wx_login").await?;
    println!("OpenID: {}", login_response.openid);
    println!("Session Key: {}", login_response.session_key);
    
    Ok(())
}
```

## API 概览

### 客户端

使用 `WechatMp::builder()` 创建统一客户端，所有 API 通过该客户端访问：

```rust
use wechat_mp_sdk::{WechatMp, types::{AppId, AppSecret}};
use std::time::Duration;

// 创建客户端
let wechat = WechatMp::builder()
    .appid(AppId::new("wx1234567890abcdef")?)
    .secret(AppSecret::new("your_secret")?)
    .build()?;

// 自定义配置
let wechat = WechatMp::builder()
    .appid(AppId::new("wx1234567890abcdef")?)
    .secret(AppSecret::new("your_secret")?)
    .base_url("https://api.weixin.qq.com")  // 默认值
    .timeout(Duration::from_secs(30))       // 默认 30 秒
    .connect_timeout(Duration::from_secs(10)) // 默认 10 秒
    .build()?;
```

Token 管理已内置于客户端，无需手动创建 `TokenManager`。

### 登录认证

```rust
// 使用 wx.login() 获取的 code 进行登录
let response = wechat.auth_login("code_from_miniprogram").await?;
println!("OpenID: {}", response.openid);
println!("Session Key: {}", response.session_key);
println!("UnionID: {:?}", response.unionid);
```

### 用户信息

```rust
// 获取用户手机号
let phone_response = wechat
    .get_phone_number("code_from_getPhoneNumber")
    .await?;
println!("Phone: {}", phone_response.phone_info.phone_number);
```

### 客服消息

```rust
use wechat_mp_sdk::api::message::{Message, TextMessage, MediaMessage};

// 发送文本消息
wechat
    .send_customer_service_message(
        "user_openid",
        Message::Text { 
            text: TextMessage::new("您好！") 
        },
    )
    .await?;

// 发送图片消息
wechat
    .send_customer_service_message(
        "user_openid",
        Message::Image { 
            image: MediaMessage::new("media_id_from_upload") 
        },
    )
    .await?;

// 发送订阅消息
use wechat_mp_sdk::api::message::SubscribeMessageOptions;
let options = SubscribeMessageOptions {
    touser: "user_openid".to_string(),
    template_id: "template_id".to_string(),
    data: /* SubscribeMessageData */,
    page: Some("pages/index/index".to_string()),
    miniprogram_state: None,
    lang: None,
};
wechat.send_subscribe_message(options).await?;
```

### 临时素材上传

```rust
// 上传临时素材
let image_data = std::fs::read("image.png")?;
let response = wechat
    .upload_temp_media(
        wechat_mp_sdk::api::MediaType::Image,
        "image.png",
        &image_data,
    )
    .await?;
println!("Media ID: {}", response.media_id);
println!("Expires in: {}s", response.expires_in);

// 下载临时素材
let bytes = wechat.get_temp_media("media_id").await?;
```

### 小程序码

```rust
use wechat_mp_sdk::api::qrcode::{
    QrcodeOptions, UnlimitQrcodeOptions,
    UrlSchemeOptions, UrlSchemeExpire, UrlLinkOptions,
    ShortLinkOptions, LineColor
};

// 获取小程序码
let options = QrcodeOptions {
    path: Some("/pages/index/index".to_string()),
    width: Some(430),
    auto_color: Some(false),
    line_color: Some(LineColor { r: 0, g: 0, b: 0 }),
    is_hyaline: Some(false),
};
let bytes = wechat.get_wxa_code(options).await?;
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
let bytes = wechat.get_wxa_code_unlimit(options).await?;

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
let scheme_url = wechat.generate_url_scheme(options).await?;

// 生成 URL Link
let options = UrlLinkOptions {
    path: Some("/pages/index/index".to_string()),
    query: Some("id=123".to_string()),
    expire_type: Some(1),
    expire_time: Some(1672531200),
    expire_interval: None,
};
let link_url = wechat.generate_url_link(options).await?;

// 生成短链接
let options = ShortLinkOptions {
    page_url: "https://example.com/page".to_string(),
};
let short_link = wechat.generate_short_link(options).await?;
```

### Access Token 管理

Token 管理已内置于 `WechatMp` 客户端，自动处理缓存、刷新和并发安全：

```rust
use std::time::Duration;

// 获取当前 Access Token（自动缓存和刷新，并发安全）
let token = wechat.get_access_token().await?;
println!("Access Token: {}", token);

// 手动失效 Token（当检测到 Token 被第三方恶意使用时）
wechat.invalidate_token().await;
```

内置 Token 管理特性：
- **自动缓存**: Token 有效期内复用，避免重复请求
- **自动刷新**: Token 过期前自动刷新（默认提前 5 分钟）
- **并发安全**: 多个并发请求共享同一 Token
- **单飞模式**: 并发请求只触发一次 API 调用
- **智能重试**: 自动重试临时性错误（如系统繁忙 -1、频率限制 45009）

### 数据解密

```rust
let session_key = "session_key_from_login";
let encrypted_data = "encrypted_data_from_miniprogram";
let iv = "iv_from_miniprogram";

// 解密用户数据
let decrypted = wechat
    .decrypt_user_data(session_key, encrypted_data, iv)?;

// 校验 watermark
wechat.verify_watermark(&decrypted)?;

// 访问解密后的数据
if let Some(open_id) = &decrypted.open_id {
    println!("OpenID: {}", open_id);
}
```

## 错误处理

SDK 采用四层错误模型，按调用顺序分为：

1. **传输层错误** (`WechatError::Http(HttpError::Reqwest)`): 网络连接、DNS 解析、超时等
2. **状态码错误** (`WechatError::Http(HttpError::Reqwest)`): HTTP 状态码非 2xx（如 400、401、403、500 等）
3. **解码错误** (`WechatError::Http(HttpError::Decode)`): 响应体 JSON 格式正确但与预期类型不匹配
4. **API 业务错误** (`WechatError::Api { code, message }`): 微信返回 errcode != 0

```rust
use wechat_mp_sdk::WechatError;

match result {
    Ok(response) => { /* 处理成功响应 */ }
    Err(WechatError::Api { code, message }) => {
        eprintln!("API 错误: {} - {}", code, message);
    }
    Err(WechatError::Http(e)) => {
        // 传输错误、非 2xx 状态码、或响应体类型不匹配
        // 可通过 wechat_mp_sdk::error::HttpError 进一步区分：
        //   HttpError::Reqwest(_) — 网络/状态码错误
        //   HttpError::Decode(_) — 响应解码错误
        eprintln!("HTTP 错误: {}", e);
    }
    Err(WechatError::Token(msg)) => {
        eprintln!("Token 错误: {}", msg);
    }
    Err(WechatError::Config(msg)) => {
        eprintln!("配置错误: {}", msg);
    }
    Err(WechatError::Crypto(msg)) => {
        eprintln!("加解密错误: {}", msg);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

### 错误处理最佳实践

- **区分传输错误和业务错误**: 非 2xx 响应码属于 `HttpError::Reqwest`，而不是 `WechatError::Api`
- **先处理网络错误，再处理业务错误**: 网络问题可能导致无法获取完整的业务错误信息
- **使用 `?` 运算符传播错误**: 错误类型会自动转换

## 类型安全

SDK 使用强类型 ID 防止参数混用，并在构造时进行严格校验：

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

### 验证规则

| 类型 | 验证规则 | 拒绝内容 |
|------|----------|----------|
| `AppId` | 以 "wx" 开头，18 字符 | 长度不符、前缀错误 |
| `AppSecret` | 非空、无控制字符 | 空字符串、全空白、控制字符 |
| `SessionKey` | 有效 base64，解码后 16 字节 | 空、空白、无效 base64、长度不符 |
| `UnionId` | 非空、无控制字符 | 空字符串、全空白、控制字符 |
| `AccessToken` | 非空、无首尾空白、无控制字符 | 空、全空白、控制字符、首尾空格 |

### 验证失败处理

验证失败时返回 `WechatError` 变体，如 `WechatError::InvalidSessionKey(...)`：

```rust
use wechat_mp_sdk::types::SessionKey;
use wechat_mp_sdk::WechatError;

let result = SessionKey::new("invalid!!base64!!!");
assert!(result.is_err());

match result {
    Err(WechatError::InvalidSessionKey(msg)) => {
        eprintln!("SessionKey 验证失败: {}", msg);
    }
    _ => {}
}
```

## 兼容性说明

本次更新包含以下兼容性变更：

### 错误模型变更

- `WechatError::Http` 现在内部封装 `HttpError` 枚举，而非直接使用 `reqwest::Error`
- 非 2xx HTTP 响应现在归类为 `HttpError::Reqwest`（传输层），而非 `WechatError::Api`（业务层）
- JSON 解码失败归类为 `HttpError::Decode`，便于区分网络错误和数据错误

### 验证规则加强

以下类型在构造时进行了更严格的校验：

- `SessionKey`: 新增 base64 格式验证和长度校验（必须解码为 16 字节）
- `AppSecret`: 新增控制字符检测
- `UnionId`: 新增控制字符检测
- `AccessToken`: 新增首尾空白和控制字符检测

如果现有代码使用了不符合新校验规则的输入，需要更新输入数据。

### 中间件管道

`WechatMp::builder().with_middleware()` 现在正确连接中间件到请求执行路径（此前为占位符）。

### 无 Panic 保证

SDK 在生产路径中移除了所有 `unwrap()` 和 `expect()` 调用，提升了运行稳定性：

- 认证中间件不再因格式错误的 URI 而 panic
- 重试中间件不再因 `max_retries=0` 而 panic

### 公共 API 保持兼容

所有公共 API 方法签名保持不变，现有代码无需修改即可编译运行。

## 从 0.1.x 迁移到 0.2.0

### 重大变更

1. **统一客户端**: `WechatMp` 替代了分离的 `WechatClient` + `TokenManager` + `Api` 模式
2. **内置 Token 管理**: `TokenManager` 不再需要手动创建和管理
3. **模块结构简化**: API 方法直接通过 `WechatMp` 实例调用

### 迁移示例

#### 旧版 (0.1.x)

```rust
use wechat_mp_sdk::{
    WechatClient, WechatClientBuilder,
    api::auth::AuthApi,
    token::TokenManager,
    types::{AppId, AppSecret},
};

let client = WechatClient::builder()
    .appid(AppId::new("wx1234567890abcdef")?)
    .secret(AppSecret::new("your_secret".to_string())?)
    .build()?;

let token_manager = TokenManager::new(client.clone());
let auth_api = AuthApi::new(client.clone());

let response = auth_api.login("code").await?;
// 使用 token_manager 处理其他 API...
```

#### 新版 (0.2.0)

```rust
use wechat_mp_sdk::{WechatMp, types::{AppId, AppSecret}};

let wechat = WechatMp::builder()
    .appid(AppId::new("wx1234567890abcdef")?)
    .secret(AppSecret::new("your_secret")?)
    .build()?;

let response = wechat.auth_login("code").await?;
// Token 自动管理，无需额外处理
```

#### 主要变化对比

| 特性 | 0.1.x | 0.2.0 |
|------|-------|-------|
| 客户端创建 | `WechatClient::builder()` | `WechatMp::builder()` |
| Token 管理 | 手动创建 `TokenManager` | 内置自动管理 |
| API 调用 | 创建独立 API 实例 | 通过 `wechat` 实例直接调用 |
| 获取 Token | `token_manager.get_token()` | `wechat.get_access_token()` |
| Token 失效 | `token_manager.invalidate()` | `wechat.invalidate_token()` |

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
