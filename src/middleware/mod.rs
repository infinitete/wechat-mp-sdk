//! Middleware components for WeChat SDK.
//!
//! This module provides middleware infrastructure using Tower patterns.
//! Middleware can be composed using `ServiceBuilder` to add cross-cutting
//! concerns like authentication, retry logic, and logging.
//!
//! ## Middleware Types
//!
//! - [`AuthMiddleware`] - Injects access_token into requests
//! - [`RetryMiddleware`] - Retries on 5xx/retryable errors
//! - [`LoggingMiddleware`] - Logs request/response information
//!
//! ## Usage
//!
//! ```ignore
//! use tower::ServiceBuilder;
//! use wechat_mp_sdk::middleware::{AuthMiddleware, RetryMiddleware, LoggingMiddleware};
//!
//! let service = ServiceBuilder::new()
//!     .layer(LoggingMiddleware::new())
//!     .layer(RetryMiddleware::new())
//!     .layer(AuthMiddleware::new())
//!     .service(inner_service);
//! ```

// Re-export tower types for convenience
pub use tower::{Layer, Service, ServiceBuilder};

mod auth;
mod logging;
mod retry;

pub use auth::{AuthMiddleware, ConfigurableAuthMiddleware, TokenInjection};
pub use logging::LoggingMiddleware;
pub use retry::RetryMiddleware;
