use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};
use tower::{Layer, Service};

use crate::api::WechatContext;
use crate::error::WechatError;
use crate::token::TokenManager;
use crate::types::{AppId, AppSecret};

use super::wechat_client::{
    WechatClient, DEFAULT_BASE_URL, DEFAULT_CONNECT_TIMEOUT_SECS, DEFAULT_TIMEOUT_SECS,
};
use super::WechatMp;

type MiddlewareFuture =
    Pin<Box<dyn Future<Output = Result<ReqwestResponse, reqwest::Error>> + Send>>;
type MiddlewareExecutor = Arc<dyn Fn(ReqwestRequest) -> MiddlewareFuture + Send + Sync>;

#[must_use]
#[derive(Default)]
pub struct WechatMpBuilder<M = ()> {
    appid: Option<AppId>,
    secret: Option<AppSecret>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    middleware: Option<M>,
}

impl<M> std::fmt::Debug for WechatMpBuilder<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WechatMpBuilder")
            .field("appid", &self.appid)
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("middleware", &self.middleware.as_ref().map(|_| ".."))
            .finish_non_exhaustive()
    }
}

impl<M> WechatMpBuilder<M> {
    pub fn appid(mut self, appid: AppId) -> Self {
        self.appid = Some(appid);
        self
    }

    pub fn secret(mut self, secret: AppSecret) -> Self {
        self.secret = Some(secret);
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    pub fn with_middleware<M2>(self, middleware: M2) -> WechatMpBuilder<M2>
    where
        M2: Layer<WechatClient> + Clone + Send + Sync + 'static,
    {
        WechatMpBuilder {
            appid: self.appid,
            secret: self.secret,
            base_url: self.base_url,
            timeout: self.timeout,
            connect_timeout: self.connect_timeout,
            middleware: Some(middleware),
        }
    }

    pub fn build(self) -> Result<WechatMp, WechatError>
    where
        M: Layer<WechatClient> + Clone + Send + Sync + 'static,
        M::Service: Service<ReqwestRequest, Response = ReqwestResponse, Error = reqwest::Error>
            + Clone
            + Send
            + Sync
            + 'static,
        <M::Service as Service<ReqwestRequest>>::Future: Send + 'static,
    {
        let appid = self
            .appid
            .ok_or_else(|| WechatError::Config("appid is required".to_string()))?;
        let secret = self
            .secret
            .ok_or_else(|| WechatError::Config("secret is required".to_string()))?;

        let base_url = self
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(WechatError::Config(format!(
                "base_url must start with http:// or https://, got: {}",
                base_url
            )));
        }

        let timeout = self
            .timeout
            .unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        let connect_timeout = self
            .connect_timeout
            .unwrap_or(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS));

        let mut client = WechatClient::builder()
            .appid(appid)
            .secret(secret)
            .base_url(base_url)
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            .build()?;

        if let Some(middleware) = self.middleware {
            let service = middleware.layer(client.clone());
            let executor = make_middleware_executor(service);
            client = client.with_middleware_executor(executor);
        }

        let client_arc = Arc::new(client);
        // Clone via deref—Arc<WechatClient> → &WechatClient → WechatClient::clone
        let token_manager = Arc::new(TokenManager::new(WechatClient::clone(&client_arc)));
        let context = Arc::new(WechatContext::new(client_arc, token_manager));

        Ok(WechatMp::from(context))
    }
}

fn make_middleware_executor<S>(service: S) -> MiddlewareExecutor
where
    S: Service<ReqwestRequest, Response = ReqwestResponse, Error = reqwest::Error>
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send + 'static,
{
    let service = Arc::new(service);

    Arc::new(move |request: ReqwestRequest| {
        let mut service = (*service).clone();
        Box::pin(async move { service.call(request).await })
    })
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use tower::{Layer, Service};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[test]
    fn test_builder_default_values() {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        let wechat = WechatMp::builder()
            .appid(appid.clone())
            .secret(secret.clone())
            .build()
            .unwrap();

        assert_eq!(wechat.appid(), appid.as_str());
    }

    #[test]
    fn test_builder_custom_values() {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        let wechat = WechatMp::builder()
            .appid(appid)
            .secret(secret)
            .base_url("https://custom.api.example.com")
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        assert_eq!(wechat.appid(), "wx1234567890abcdef");
    }

    #[tokio::test]
    async fn test_middleware_configured_and_executes() {
        #[derive(Clone)]
        struct FlagLayer {
            flag: Arc<AtomicBool>,
        }

        impl Layer<WechatClient> for FlagLayer {
            type Service = FlagService;

            fn layer(&self, inner: WechatClient) -> Self::Service {
                FlagService {
                    inner,
                    flag: Arc::clone(&self.flag),
                }
            }
        }

        #[derive(Clone)]
        struct FlagService {
            inner: WechatClient,
            flag: Arc<AtomicBool>,
        }

        impl Service<ReqwestRequest> for FlagService {
            type Response = ReqwestResponse;
            type Error = reqwest::Error;
            type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: ReqwestRequest) -> Self::Future {
                self.flag.store(true, Ordering::SeqCst);
                let mut inner = self.inner.clone();
                Box::pin(async move { inner.call(req).await })
            }
        }

        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();
        let middleware_invoked = Arc::new(AtomicBool::new(false));
        let layer = FlagLayer {
            flag: Arc::clone(&middleware_invoked),
        };

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sns/jscode2session"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "openid": "test_openid",
                "session_key": "test_session_key",
                "unionid": null,
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let wechat = WechatMp::builder()
            .appid(appid)
            .secret(secret)
            .base_url(mock_server.uri())
            .with_middleware(layer)
            .build()
            .unwrap();

        let _ = wechat.auth_login("mock_js_code").await.unwrap();

        assert!(middleware_invoked.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_builder_with_logging_middleware_builds() {
        let appid = AppId::new("wx1234567890abcdef").unwrap();
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sns/jscode2session"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "openid": "test_openid",
                "session_key": "test_session_key",
                "unionid": null,
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&mock_server)
            .await;

        let wechat = WechatMp::builder()
            .appid(appid)
            .secret(secret)
            .base_url(mock_server.uri())
            .with_middleware(crate::middleware::LoggingMiddleware::new())
            .build()
            .unwrap();

        let result = wechat.auth_login("mock_js_code").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_appid() {
        let secret = AppSecret::new("secret1234567890ab").unwrap();

        let result = WechatMp::builder().secret(secret).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_missing_secret() {
        let appid = AppId::new("wx1234567890abcdef").unwrap();

        let result = WechatMp::builder().appid(appid).build();

        assert!(result.is_err());
    }
}
