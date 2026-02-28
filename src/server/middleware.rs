use crate::core::api_key::ApiKey;
use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::{
    collections::{HashMap, VecDeque},
    future::{ready, Ready},
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct LoggingMiddleware {
    server_logger: Arc<crate::server::logging::ServerLogger>,
}

impl LoggingMiddleware {
    pub fn new(server_logger: Arc<crate::server::logging::ServerLogger>) -> Self {
        Self { server_logger }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddlewareService<S>;
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggingMiddlewareService {
            service,
            server_logger: self.server_logger.clone(),
        }))
    }
}

pub struct LoggingMiddlewareService<S> {
    service: S,
    server_logger: Arc<crate::server::logging::ServerLogger>,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let server_logger = self.server_logger.clone();

        let ip = {
            let connection_info = req.connection_info();
            connection_info
                .realip_remote_addr()
                .or_else(|| connection_info.peer_addr())
                .unwrap_or("unknown")
                .split(':')
                .next()
                .unwrap_or("unknown")
                .to_string()
        };

        let path = req.path().to_string();
        let method = req.method().to_string();
        let query_string = req.query_string().to_string();

        let suspicious = is_suspicious_path(&path);

        if suspicious {
            let logger_clone = server_logger.clone();
            let ip_clone = ip.clone();
            let path_clone = path.clone();
            tokio::spawn(async move {
                let _ = logger_clone
                    .log_security_alert(
                        &ip_clone,
                        "Suspicious Request",
                        &format!("Suspicious path: {}", path_clone),
                    )
                    .await;
            });
        }

        let headers: std::collections::HashMap<String, String> = req
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                let header_name = name.as_str().to_lowercase();
                if !["authorization", "cookie", "x-api-key"].contains(&header_name.as_str()) {
                    value
                        .to_str()
                        .ok()
                        .map(|v| (name.as_str().to_string(), v.to_string()))
                } else {
                    Some((name.as_str().to_string(), "[FILTERED]".to_string()))
                }
            })
            .collect();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let response_time = start_time.elapsed().as_millis() as u64;
            let status = res.status().as_u16();
            let bytes_sent = res
                .response()
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Clone for analytics tracking (values move into the log entry below)
            let analytics_path = path.clone();
            let analytics_ip = ip.clone();
            let analytics_ua = headers.get("user-agent").cloned().unwrap_or_default();

            let entry = crate::server::logging::ServerLogEntry {
                timestamp: chrono::Local::now()
                    .format("%Y-%m-%d %H:%M:%S%.3f")
                    .to_string(),
                timestamp_unix: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                event_type: crate::server::logging::LogEventType::Request,
                ip_address: ip,
                user_agent: headers.get("user-agent").cloned(),
                method,
                path,
                status_code: Some(status),
                response_time_ms: Some(response_time),
                bytes_sent: Some(bytes_sent),
                referer: headers.get("referer").cloned(),
                query_string: if query_string.is_empty() {
                    None
                } else {
                    Some(query_string)
                },
                headers,
                session_id: None,
            };

            if let Err(e) = server_logger.write_log_entry(entry).await {
                log::error!("Failed to log request: {}", e);
            }

            crate::server::analytics::track_request("", &analytics_path, &analytics_ip, &analytics_ua);

            Ok(res)
        })
    }
}

fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn is_suspicious_path(path: &str) -> bool {
    let decoded = percent_decode(path);
    let normalized = decoded.replace('\\', "/");
    let lower = normalized.to_lowercase();

    normalized.contains("..")
        || lower.contains("<script")
        || lower.contains("union select")
        || lower.contains("drop table")
        || path.len() > 1000
}

// =============================================================================
// API Key Authentication Middleware
// =============================================================================

#[derive(Clone)]
pub struct ApiKeyAuth {
    api_key: ApiKey,
}

impl ApiKeyAuth {
    pub fn new(api_key: ApiKey) -> Self {
        Self { api_key }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyAuthService<S>;
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyAuthService {
            service,
            api_key: self.api_key.clone(),
        }))
    }
}

pub struct ApiKeyAuthService<S> {
    service: S,
    api_key: ApiKey,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();

        let is_public_asset = path == "/.rss/"
            || path == "/.rss/_reset.css"
            || path == "/.rss/style.css"
            || path == "/.rss/favicon.svg"
            || path.starts_with("/.rss/js/")
            || path.starts_with("/.rss/fonts/")
            || path == "/ws/hot-reload";

        let needs_auth =
            (path.starts_with("/api/") || path.starts_with("/.rss/") || path.starts_with("/ws/"))
                && path != "/api/health"
                && !path.starts_with("/api/acme/")
                && !path.starts_with("/api/analytics")
                && !path.starts_with("/.well-known/")
                && !is_public_asset;

        // Skip auth if not needed or no key configured
        if !needs_auth || self.api_key.is_empty() {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        // Check X-API-Key header
        let header_key = req
            .headers()
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Check ?api_key= query parameter
        let query_key = req
            .query_string()
            .split('&')
            .find_map(|param| param.strip_prefix("api_key="))
            .map(|s| s.to_string());

        let provided_key = header_key.or(query_key);

        let is_valid = provided_key
            .as_deref()
            .map(|k| self.api_key.verify(k))
            .unwrap_or(false);
        if is_valid {
            let fut = self.service.call(req);
            Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
        } else {
            let response = HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Valid API key required. Provide via X-API-Key header or ?api_key= query parameter."
                }));
            Box::pin(async move { Ok(req.into_response(response).map_into_right_body()) })
        }
    }
}

// =============================================================================
// Rate Limiter Middleware
// =============================================================================

#[derive(Clone)]
pub struct RateLimiter {
    max_rps: u32,
    enabled: bool,
    clients: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl RateLimiter {
    pub fn new(max_rps: u32, enabled: bool) -> Self {
        Self {
            max_rps,
            enabled,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterService<S>;
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterService {
            service,
            max_rps: self.max_rps,
            enabled: self.enabled,
            clients: self.clients.clone(),
        }))
    }
}

pub struct RateLimiterService<S> {
    service: S,
    max_rps: u32,
    enabled: bool,
    clients: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only rate-limit /api/* paths
        if !self.enabled || !req.path().starts_with("/api/") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        let ip = {
            let connection_info = req.connection_info();
            connection_info
                .realip_remote_addr()
                .or_else(|| connection_info.peer_addr())
                .unwrap_or("unknown")
                .split(':')
                .next()
                .unwrap_or("unknown")
                .to_string()
        };

        let now = Instant::now();
        let one_second_ago = now - std::time::Duration::from_secs(1);

        let is_limited = if let Ok(mut clients) = self.clients.lock() {
            let timestamps = clients.entry(ip).or_insert_with(VecDeque::new);

            // Remove entries older than 1 second
            while timestamps.front().is_some_and(|t| *t < one_second_ago) {
                timestamps.pop_front();
            }

            if timestamps.len() >= self.max_rps as usize {
                true
            } else {
                timestamps.push_back(now);
                false
            }
        } else {
            false // If lock fails, allow the request
        };

        if is_limited {
            let response = HttpResponse::TooManyRequests()
                .insert_header(("Retry-After", "1"))
                .json(serde_json::json!({
                    "error": "Too Many Requests",
                    "message": "Rate limit exceeded. Try again later.",
                    "retry_after": 1
                }));
            Box::pin(async move { Ok(req.into_response(response).map_into_right_body()) })
        } else {
            let fut = self.service.call(req);
            Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- percent_decode tests ---

    #[test]
    fn test_percent_decode_plain() {
        assert_eq!(percent_decode("/api/status"), "/api/status");
    }

    #[test]
    fn test_percent_decode_encoded_slash() {
        assert_eq!(percent_decode("%2F"), "/");
    }

    #[test]
    fn test_percent_decode_dot_dot() {
        assert_eq!(percent_decode("%2e%2e"), "..");
    }

    #[test]
    fn test_percent_decode_mixed() {
        assert_eq!(percent_decode("/foo%2Fbar%2E%2E%2Fbaz"), "/foo/bar../baz");
    }

    #[test]
    fn test_percent_decode_incomplete_sequence() {
        assert_eq!(percent_decode("abc%2"), "abc%2");
    }

    #[test]
    fn test_percent_decode_invalid_hex() {
        assert_eq!(percent_decode("%ZZ"), "%ZZ");
    }

    #[test]
    fn test_percent_decode_empty() {
        assert_eq!(percent_decode(""), "");
    }

    #[test]
    fn test_percent_decode_script_tag() {
        assert_eq!(percent_decode("%3Cscript%3E"), "<script>");
    }

    // --- is_suspicious_path tests ---

    #[test]
    fn test_suspicious_path_traversal() {
        assert!(is_suspicious_path("/../etc/passwd"));
        assert!(is_suspicious_path("/foo/../../etc/shadow"));
    }

    #[test]
    fn test_suspicious_path_encoded_traversal() {
        assert!(is_suspicious_path("/%2e%2e/etc/passwd"));
        assert!(is_suspicious_path("/%2E%2E/secret"));
    }

    #[test]
    fn test_suspicious_path_backslash_traversal() {
        assert!(is_suspicious_path("/foo\\..\\etc\\passwd"));
    }

    #[test]
    fn test_suspicious_path_script_injection() {
        assert!(is_suspicious_path("/<script>alert(1)</script>"));
        assert!(is_suspicious_path("/%3Cscript%3Ealert(1)"));
    }

    #[test]
    fn test_suspicious_path_sql_injection() {
        assert!(is_suspicious_path("/api?q=1 UNION SELECT * FROM users"));
        assert!(is_suspicious_path("/api?q=DROP TABLE users"));
    }

    #[test]
    fn test_suspicious_path_too_long() {
        let long_path = "/".to_string() + &"a".repeat(1001);
        assert!(is_suspicious_path(&long_path));
    }

    #[test]
    fn test_safe_paths() {
        assert!(!is_suspicious_path("/"));
        assert!(!is_suspicious_path("/api/status"));
        assert!(!is_suspicious_path("/index.html"));
        assert!(!is_suspicious_path("/.rss/style.css"));
        assert!(!is_suspicious_path("/api/logs?offset=100"));
        assert!(!is_suspicious_path("/ws/hot-reload"));
    }

    #[test]
    fn test_safe_path_with_dots_in_filename() {
        assert!(!is_suspicious_path("/file.name.html"));
        assert!(!is_suspicious_path("/.rss/favicon.svg"));
    }
}
