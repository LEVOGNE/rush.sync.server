use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    sync::Arc,
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

        let suspicious = path.contains("..")
            || path.contains("<script")
            || path.contains("sql")
            || path.len() > 1000;

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

            Ok(res)
        })
    }
}
