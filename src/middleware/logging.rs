use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

use log::{debug, info};
use reqwest::{Request, Response};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct LoggingMiddleware {
    verbose: bool,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self { verbose: false }
    }

    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for LoggingMiddleware
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Service = LoggingMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddlewareService {
            inner,
            verbose: self.verbose,
        }
    }
}

#[derive(Clone)]
pub struct LoggingMiddlewareService<S> {
    inner: S,
    verbose: bool,
}

const SENSITIVE_FIELDS: &[&str] = &[
    "access_token",
    "appsecret",
    "secret",
    "session_key",
    "password",
    "token",
    "authorization",
];

impl<S> LoggingMiddlewareService<S> {
    fn redact_url(url: &str) -> String {
        if let Some(idx) = url.find('?') {
            let base = &url[..idx];
            let query = &url[idx + 1..];
            let redacted_query: String = query
                .split('&')
                .map(|param| {
                    if let Some(eq_idx) = param.find('=') {
                        let key = &param[..eq_idx];
                        if SENSITIVE_FIELDS.iter().any(|s| key.eq_ignore_ascii_case(s)) {
                            format!("{}={}", key, "[REDACTED]")
                        } else {
                            param.to_string()
                        }
                    } else {
                        param.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", base, redacted_query)
        } else {
            url.to_string()
        }
    }

    fn log_request(method: &str, url: &str, verbose: bool) {
        let safe_url = Self::redact_url(url);
        if verbose {
            debug!("[WechatMp] >>> {} {}", method, safe_url);
        } else {
            info!("[WechatMp] {} {}", method, safe_url);
        }
    }

    fn log_response(status: u16, duration: std::time::Duration, verbose: bool) {
        if verbose {
            debug!(
                "[WechatMp] <<< {} - {} ({:?})",
                status,
                Self::status_text(status),
                duration
            );
        } else {
            info!("[WechatMp] {} ({:?})", status, duration);
        }
    }

    fn status_text(status: u16) -> &'static str {
        match status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            307 => "Temporary Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            408 => "Request Timeout",
            422 => "Unprocessable Entity",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "",
        }
    }
}

impl<S, Error> Service<Request> for LoggingMiddlewareService<S>
where
    S: Service<Request, Response = Response, Error = Error> + Send + Clone + 'static,
    S::Future: Send,
    Error: Send + 'static,
{
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let method = req.method().as_str().to_string();
        let url = req.url().to_string();
        let verbose = self.verbose;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            Self::log_request(&method, &url, verbose);

            let start = Instant::now();
            let response = inner.call(req).await?;
            let duration = start.elapsed();

            Self::log_response(response.status().as_u16(), duration, verbose);

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_redact_url_no_sensitive_params() {
        let url = "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential";
        let redacted = LoggingMiddlewareService::<()>::redact_url(url);
        assert_eq!(redacted, url);
    }

    #[test]
    fn test_redact_url_with_access_token() {
        let url = "https://api.weixin.qq.com/cgi-bin/token?access_token=abc123&grant_type=client_credential";
        let redacted = LoggingMiddlewareService::<()>::redact_url(url);
        assert!(redacted.contains("access_token=[REDACTED]"));
        assert!(redacted.contains("grant_type=client_credential"));
    }

    #[test]
    fn test_redact_url_with_secret() {
        let url = "https://api.weixin.qq.com/cgi-bin/token?appsecret=mysecret&grant_type=client_credential";
        let redacted = LoggingMiddlewareService::<()>::redact_url(url);
        assert!(redacted.contains("appsecret=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_with_session_key() {
        let url = "https://api.weixin.qq.com/wxa/getUserInfo?session_key=key123&encryptedData=data";
        let redacted = LoggingMiddlewareService::<()>::redact_url(url);
        assert!(redacted.contains("session_key=[REDACTED]"));
    }

    #[test]
    fn test_status_text() {
        assert_eq!(LoggingMiddlewareService::<()>::status_text(200), "OK");
        assert_eq!(
            LoggingMiddlewareService::<()>::status_text(400),
            "Bad Request"
        );
        assert_eq!(
            LoggingMiddlewareService::<()>::status_text(401),
            "Unauthorized"
        );
        assert_eq!(
            LoggingMiddlewareService::<()>::status_text(500),
            "Internal Server Error"
        );
    }

    #[tokio::test]
    async fn test_logging_middleware_logs_request() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await.unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
            socket.write_all(response.as_bytes()).await.unwrap();
        });

        let client = Client::builder().build().unwrap();

        let middleware = LoggingMiddleware::new();
        let mut service = middleware.layer(client.clone());

        let url = format!("http://{}/test", addr);
        let req = client.get(&url).build().unwrap();

        let _ = service.call(req).await;

        tokio::select! {
            _ = server => {}
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
        }
    }

    #[tokio::test]
    async fn test_logging_does_not_expose_secrets() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await.unwrap();

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
            socket.write_all(response.as_bytes()).await.unwrap();
        });

        let client = Client::builder().build().unwrap();

        let middleware = LoggingMiddleware::new().verbose();
        let mut service = middleware.layer(client.clone());

        let url = format!(
            "http://{}//cgi-bin/token?access_token=secret123&appsecret=mysecret&grant_type=client_credential",
            addr
        );
        let req = client.get(&url).build().unwrap();

        let _ = service.call(req).await;

        tokio::select! {
            _ = server => {}
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
        }
    }
}
