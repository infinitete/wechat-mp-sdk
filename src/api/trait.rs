//! WeChat API trait and context
//!
//! Provides the base trait and context for all WeChat API implementations.

use std::sync::Arc;

use crate::client::WechatClient;
use crate::token::TokenManager;

/// Context holding shared resources for WeChat API implementations.
///
/// Contains references to the HTTP client and token manager that
/// API implementations need to make requests.
#[derive(Clone)]
pub struct WechatContext {
    /// The WeChat HTTP client for making API requests
    pub(crate) client: Arc<WechatClient>,
    /// The token manager for access token lifecycle
    pub(crate) token_manager: Arc<TokenManager>,
}

impl std::fmt::Debug for WechatContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WechatContext")
            .field("client", &"WechatClient { .. }")
            .field("token_manager", &"TokenManager { .. }")
            .finish()
    }
}

impl WechatContext {
    /// Create a new WechatContext
    pub fn new(client: Arc<WechatClient>, token_manager: Arc<TokenManager>) -> Self {
        Self {
            client,
            token_manager,
        }
    }

    /// Get a reference to the WeChat HTTP client.
    pub fn client(&self) -> &WechatClient {
        &self.client
    }

    /// Get a reference to the token manager.
    pub fn token_manager(&self) -> &TokenManager {
        &self.token_manager
    }
}

/// Trait for WeChat API implementations.
///
/// All API modules should implement this trait to provide
/// access to the shared context.
pub trait WechatApi: Send + Sync {
    /// Get a reference to the WeChat context
    fn context(&self) -> &WechatContext;

    /// Get the name of this API for logging and error context.
    ///
    /// Implementors should override this to return a descriptive name
    /// (e.g., "auth", "user", "customer_service").
    fn api_name(&self) -> &'static str {
        "unknown"
    }
}
