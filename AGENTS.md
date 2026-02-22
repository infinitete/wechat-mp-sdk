# AGENTS.md — wechat-mp-sdk

WeChat Mini Program server SDK for Rust. Library crate, edition 2021, MSRV 1.70.
Covers **128 endpoints** across 24 API categories.

---

## Build / Lint / Test

```bash
cargo check                                              # Fast type check
cargo build                                              # Debug build
cargo fmt                                                # Format (MUST run before commit)
cargo fmt --check                                        # Format check only
cargo clippy --all-targets --all-features -- -D warnings # Lint (warnings = errors)
cargo test                                               # All tests (unit + integration)
cargo test -- --nocapture                                # Tests with stdout
cargo test test_name                                     # Single test by name substring
cargo test tests::module_name                            # Tests in a module
cargo test --test mock_tests                             # Single integration test file
cargo test --doc                                         # Doc tests only
cargo test -- --ignored                                  # Ignored tests only
cargo doc --open                                         # Build & browse docs
```

### Pre-commit hook

Runs `cargo fmt --check` → `cargo clippy` → `cargo test`. Install with `./setup-hooks.sh`.

---

## Architecture

```
src/
├── lib.rs                # Crate root — re-exports public API
├── error.rs              # WechatError + HttpError (thiserror)
├── token.rs              # TokenManager — auto-cache, refresh, single-flight
├── types/
│   ├── ids.rs            # Newtype IDs: AppId, AppSecret, OpenId, UnionId, SessionKey, AccessToken
│   └── watermark.rs      # Watermark verification
├── client/
│   ├── wechat_client.rs  # WechatClient + WechatClientBuilder — HTTP layer (reqwest)
│   ├── wechat_mp.rs      # WechatMp — unified facade (128 API methods)
│   └── builder.rs        # WechatMpBuilder
├── api/
│   ├── trait.rs          # WechatApi trait + WechatContext (shared client + token_manager)
│   ├── common.rs         # Shared response/pagination types
│   ├── auth.rs           # Login, stable token, session checks
│   ├── user.rs           # Phone number, user info, encryption keys
│   ├── customer_service.rs # Customer service messages
│   ├── subscribe.rs      # Subscribe messages + template management
│   ├── qrcode.rs         # Mini Program codes, URL Scheme/Link, short links
│   ├── security.rs       # Content safety (text/image)
│   ├── analytics.rs      # Visit trends, retention, user profiles
│   └── ...               # 15+ more API modules (operations, ocr, cloud, live, etc.)
├── middleware/
│   ├── auth.rs           # Token injection middleware (Tower)
│   ├── retry.rs          # Retry middleware
│   └── logging.rs        # Request/response logging
└── crypto/
    └── aes.rs            # AES-128-CBC decryption + watermark
```

### Key patterns

- **Builder pattern**: `WechatMp::builder().appid(...).secret(...).build()?`
- **Newtype IDs**: `AppId`, `OpenId`, etc. validate on construction — compile-time misuse prevention
- **API trait**: Each API module struct holds `Arc<WechatContext>`, implements `WechatApi` trait
- **Single-flight token**: `TokenManager` merges concurrent refresh calls via `RwLock` + `Notify`
- **Tower middleware**: Auth, retry, logging composed via `ServiceBuilder`
- **Async everywhere**: All I/O is tokio-based; `WechatClient::get/post` use `DeserializeOwned`

---

## Code Style

### Formatting
- `cargo fmt` before every commit — line width 100, 4-space indent, K&R braces

### Imports (group with blank lines between)
```rust
// 1. std
use std::sync::Arc;

// 2. External crates
use serde::{Deserialize, Serialize};
use thiserror::Error;

// 3. Crate-internal
use crate::error::WechatError;

// 4. Parent/sibling
use super::common::ApiResponseBase;
```

### Naming
| Item              | Convention           | Example                        |
|-------------------|----------------------|--------------------------------|
| Crates/Modules    | snake_case           | `wechat_mp_sdk`, `auth`       |
| Types/Traits      | PascalCase           | `WechatMp`, `WechatApi`       |
| Functions/Vars    | snake_case           | `get_phone_number()`           |
| Constants         | SCREAMING_SNAKE_CASE | `MAX_RETRIES`                  |
| Type params       | Single uppercase     | `T`, `E`                       |

### Error handling
- Use `thiserror` for all error enums (see `WechatError` in `src/error.rs`)
- Propagate with `?` — never `unwrap()`/`expect()` in library code (OK in tests)
- Every API response checks `errcode != 0` → returns `WechatError::Api { code, message }`
- Binary responses (QR images, media) check content-type to distinguish success/error JSON

### Type safety
- **Newtype pattern** for all IDs — validated on `::new()`, `::new_unchecked()` for trusted input
- Sensitive values (`AppSecret`, `SessionKey`) redact in `Debug`/`Display` impls
- `#[non_exhaustive]` on public response structs for future compatibility
- `#[serde(default)]` on optional response fields — WeChat API may omit fields

### Documentation
```rust
/// Brief one-line description.
///
/// # Arguments
/// * `js_code` - The code from wx.login()
///
/// # Errors
/// Returns `WechatError::Api` if errcode is non-zero.
pub async fn login(&self, js_code: &str) -> Result<LoginResponse, WechatError> { ... }
```

---

## Testing

### Unit tests
Inline `#[cfg(test)] mod tests` in each module. Pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_parses_success() {
        let json = r#"{"openid":"oABC","errcode":0,"errmsg":"ok"}"#;
        let resp: LoginResponse = serde_json::from_str(json).unwrap();
        assert!(resp.is_success());
    }
}
```

### Integration tests (`tests/`)
Use **wiremock** for HTTP mocking — no real network calls:

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};

#[tokio::test]
async fn test_mock_login() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sns/jscode2session"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({...})))
        .mount(&mock_server).await;
    let client = create_test_client(&mock_server).await;
    // ... assert
}
```

Test fixtures: `AppId("wx1234567890abcdef")`, `AppSecret("test_secret_12345")`.

### Adding a new API module

1. Create `src/api/{name}.rs` — define request/response structs + `{Name}Api` struct
2. Implement `WechatApi` trait (return `&self.context` and `api_name()`)
3. Register in `src/api/mod.rs` — `pub mod {name}; pub use {name}::{...};`
4. Add facade methods to `src/client/wechat_mp.rs` (WechatMp delegates to XxxApi)
5. Add unit tests inline, integration tests in `tests/`

---

## Dependencies

| Crate             | Purpose                              |
|-------------------|--------------------------------------|
| `reqwest`         | HTTP client (json + multipart)       |
| `tokio`           | Async runtime                        |
| `serde` / `serde_json` | Serialization                  |
| `thiserror`       | Error derive macros                  |
| `tower`           | Middleware composition               |
| `aes` / `cbc`     | AES-128-CBC decryption               |
| `base64`          | Base64 encoding/decoding             |
| `wiremock` (dev)  | HTTP mock server for tests           |

## Features

- `default = ["rustls-tls"]` — rustls TLS backend
- `native-tls` — system native TLS backend
