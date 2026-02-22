# wechat-mp-sdk

[![Crates.io](https://img.shields.io/crates/v/wechat-mp-sdk.svg)](https://crates.io/crates/wechat-mp-sdk)
[![Documentation](https://docs.rs/wechat-mp-sdk/badge.svg)](https://docs.rs/wechat-mp-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

微信小程序服务端 SDK for Rust。

当前版本：`0.2.0`

## 最近更新

- 统一重试判定逻辑：中间件与 Token 管理均基于 `is_transient()` 判断是否可重试
- 修复回退抖动实现：采用确定性退避 + 轻量随机抖动，避免并发请求同节奏重试
- 增补回归测试：覆盖 `HttpError::Decode` 不重试与抖动分布行为

## 功能特性

覆盖微信小程序服务端 **128 个接口**，跨 24 个功能分类：

- 登录认证与 Session 管理
- Access Token 自动管理（内置于客户端，支持并发安全、单飞模式）
- OpenAPI 配额与调用管理
- 用户信息获取、手机号获取与数据解密
- 小程序码/二维码生成（含 URL Scheme、URL Link、短链接）
- 客服消息发送与临时素材管理
- 订阅消息发送与模板管理
- 内容安全检测（文本、图片异步检测）
- 数据分析（访问趋势、留存、用户画像等）
- 运营中心（日志、反馈、灰度发布等）
- 图像处理与 OCR 识别
- 插件管理、附近小程序、云开发
- 直播、硬件/IoT、即时配送、物流
- 服务市场、生物认证、人脸核身、微信搜索、广告、微信客服


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
- **精确重试边界**: 对 `HttpError::Decode` 等非瞬时错误立即返回，不做无效重试

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

> 注：对媒体下载接口而言，若微信返回 JSON 错误体（例如 `errcode`/`errmsg`），SDK 会按业务错误返回 `WechatError::Api`。

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
- **只对瞬时错误重试**: 可通过 `error.is_transient()` 统一判断是否应该重试
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



## 完整 API 覆盖

共 **128 个接口**，跨 24 个分类（1 个已废弃接口不计入）。

| 分类 | 接口数 | 内容 |
|------|--------|------|
| 登录认证 | 3 | code2Session、校验/重置 SessionKey |
| Access Token | 2 | 获取普通/稳定 Token |
| OpenAPI 管理 | 8 | 配额查询、清除、RID 查询、回调检测、IP 查询 |
| 安全 | 3 | 文本安全检测、图片异步检测、用户风险等级 |
| 用户信息 | 5 | 手机号、加密数据校验、加密密鑰、UnionID |
| 二维码/链接 | 9 | 小程序码、二维码、Scheme、URL Link、短链接、NFC |
| 客服消息 | 4 | 发送消息、输入状态、临时素材上传/下载 |
| 订阅消息 | 10 | 发送、模板增删查、分类、用户通知设置 |
| 数据分析 | 11 | 日/周/月访问趋势、留存、页面、分布、用户画像、性能 |
| 运营中心 | 10 | 域名信息、实时日志、反馈、JS 错误、灰度发布 |
| 图像/OCR | 8 | AI 裁剪、扫码、印刷文字、行驶证、驾驶证、身份证、银行卡、营业执照 |
| 插件管理 | 2 | 申请/管理插件 |
| 附近小程序 | 4 | 增删查 POI、显示状态 |
| 云开发 | 10 | 云函数、数据库 CRUD、文件上传/下载/删除、发送短信（1 已废弃） |
| 直播 | 9 | 房间增删改、商品管理、推送消息、粉丝查询 |
| 硬件/IoT | 6 | 设备消息、SN 票据、设备组管理 |
| 即时配送 | 5 | 配送商查询、预下单/取消、下单/取消 |
| 物流 | 6 | 账号绑定、快递公司查询、运单增查、路径查询 |
| 服务市场 | 1 | 调用服务 |
| 生物认证 | 1 | 验证签名 |
| 人脸核身 | 2 | 获取核身 ID、查询核身结果 |
| 微信搜索 | 1 | 提交页面 |
| 广告 | 4 | 用户行为上报、行为集管理 |
| 微信客服 | 3 | 客服绑定/解绑、查询绑定 |

> 完整接口列表及实现状态详见 `src/api/endpoint_inventory.rs`。

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
