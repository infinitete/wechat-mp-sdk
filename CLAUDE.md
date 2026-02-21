# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`wechat-mp-sdk` — 微信小程序服务端 SDK for Rust (library crate, edition 2021, MSRV 1.70)。

## Common Commands

```bash
cargo check                                              # 快速类型检查
cargo build                                              # 编译
cargo test                                               # 运行所有测试
cargo test test_name                                     # 按名称匹配运行单个测试
cargo test --doc                                         # 仅文档测试
cargo fmt                                                # 格式化（提交前必须执行）
cargo clippy --all-targets --all-features -- -D warnings # lint（warnings 视为 error）
cargo doc --open                                         # 构建并打开文档
```

Pre-commit 检查项：`cargo fmt` → `cargo clippy -- -D warnings` → `cargo test` → `cargo doc`。项目配有 git hooks（运行 `./setup-hooks.sh` 安装）。

## Architecture

```
src/
├── lib.rs          # 入口，re-export 公共 API
├── client.rs       # WechatClient / WechatClientBuilder — HTTP 请求封装（reqwest）
├── token.rs        # TokenManager — access_token 自动缓存与刷新
├── error.rs        # WechatError（thiserror 派生）
├── types/
│   ├── ids.rs      # 强类型 newtype：AppId, AppSecret, OpenId, UnionId, SessionKey, AccessToken
│   └── watermark.rs
├── api/
│   ├── auth.rs     # AuthApi — code2Session 登录
│   ├── user.rs     # UserApi — 用户信息、手机号获取
│   ├── message.rs  # MessageApi — 客服消息、订阅消息、临时素材
│   └── qrcode.rs   # QrcodeApi — 小程序码、URL Scheme/Link、短链接
└── crypto/
    └── aes.rs      # AES-128-CBC 解密 + watermark 校验
```

### 核心设计模式

- **Builder 模式**：`WechatClientBuilder` 构建客户端，支持自定义 base_url 和超时（默认 30s/10s connect）。
- **Newtype 强类型**：`AppId`、`OpenId` 等 ID 类型在构造时校验（如 AppId 必须 `wx` 开头、18 字符），编译期防止 ID 混用。
- **Single-flight 合并请求**：`TokenManager` 用 `RwLock` + `Mutex` + `Notify` 实现并发 token 刷新请求合并，避免重复调用微信 API。含指数退避重试（可重试错误码：-1 系统繁忙、45009 频率限制）。
- **所有 I/O 均为 async**：基于 tokio，`WechatClient::get/post` 使用泛型约束 `DeserializeOwned` / `Serialize`。

### 错误体系

`WechatError` 枚举：`Http` | `Json` | `Api { code, message }` | `Token` | `Config` | `Signature` | `Crypto` | `NotSupported`。库代码禁止 `unwrap()`/`expect()`，测试中可用。

## Code Style

- `cargo fmt` 强制格式化，行宽 100 字符
- imports 分组：std → 外部 crate → `crate::` → `super::`，组间空行
- 用 `?` 传播错误，用 `thiserror` 定义错误类型
- 类型名 PascalCase，函数/变量 snake_case，常量 SCREAMING_SNAKE_CASE

## Dependencies

运行时：`tokio`(full)、`reqwest`(json, rustls-tls)、`serde`/`serde_json`、`aes`/`cbc`、`base64`、`thiserror`

开发：`wiremock`（HTTP mock）、`tokio-test`、`futures`
