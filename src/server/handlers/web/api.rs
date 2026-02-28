use super::ServerDataWithConfig;
use crate::server::{config, logging::ServerLogger};
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    message: String,
    from: String,
    timestamp: String,
    id: u32,
}

pub async fn status_handler(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let server_dir = format!("www/{}-[{}]", data.server.name, data.server.port);

    Ok(HttpResponse::Ok().json(json!({
        "status": "running",
        "server_id": data.server.id,
        "server_name": data.server.name,
        "port": data.server.port,
        "proxy_port": data.proxy_https_port,
        "server": config::get_server_name(),
        "version": config::get_server_version(),
        "uptime_seconds": uptime,
        "static_files": true,
        "template_system": true,
        "hot_reload": true,
        "websocket_endpoint": "/ws/hot-reload",
        "server_directory": server_dir,
        "log_file": format!(".rss/servers/{}-[{}].log", data.server.name, data.server.port),
        "certificate_file": format!(".rss/certs/{}-{}.cert", data.server.name, data.server.port),
        "private_key_file": format!(".rss/certs/{}-{}.key", data.server.name, data.server.port),
        "urls": {
            "http": format!("http://127.0.0.1:{}", data.server.port),
            "proxy": format!("https://{}.localhost:{}", data.server.name, data.proxy_https_port)
        }
    })))
}

pub async fn info_handler(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let server_dir = format!("www/{}-[{}]", data.server.name, data.server.port);

    Ok(HttpResponse::Ok().json(json!({
        "name": "Rush Sync Server",
        "version": config::get_server_version(),
        "server_id": data.server.id,
        "server_name": data.server.name,
        "port": data.server.port,
        "proxy_port": data.proxy_https_port,
        "static_files_enabled": true,
        "template_system": "enabled",
        "hot_reload_enabled": true,
        "websocket_url": format!("ws://127.0.0.1:{}/ws/hot-reload", data.server.port),
        "server_directory": server_dir,
        "certificate": {
            "cert_file": format!(".rss/certs/{}-{}.cert", data.server.name, data.server.port),
            "key_file": format!(".rss/certs/{}-{}.key", data.server.name, data.server.port),
            "common_name": format!("{}.localhost", data.server.name)
        },
        "urls": {
            "http": format!("http://127.0.0.1:{}", data.server.port),
            "proxy": format!("https://{}.localhost:{}", data.server.name, data.proxy_https_port),
            "websocket": format!("ws://127.0.0.1:{}/ws/hot-reload", data.server.port)
        },
        "endpoints": [
            { "path": "/", "method": "GET", "description": "Static files from server directory", "type": "static" },
            { "path": "/.rss/favicon.svg", "method": "GET", "description": "SVG favicon", "type": "static" },
            { "path": "/api/status", "method": "GET", "description": "Server status", "type": "api" },
            { "path": "/api/info", "method": "GET", "description": "API information", "type": "api" },
            { "path": "/api/metrics", "method": "GET", "description": "Server metrics", "type": "api" },
            { "path": "/api/stats", "method": "GET", "description": "Request statistics", "type": "api" },
            { "path": "/api/logs", "method": "GET", "description": "Live server logs", "type": "api" },
            { "path": "/api/logs/raw", "method": "GET", "description": "Raw log data (JSON)", "type": "api" },
            { "path": "/api/health", "method": "GET", "description": "Health check", "type": "api" },
            { "path": "/ws/hot-reload", "method": "GET", "description": "WebSocket hot reload", "type": "websocket" }
        ]
    })))
}

pub async fn metrics_handler(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let server_dir = format!("www/{}-[{}]", data.server.name, data.server.port);
    let log_file_size = if let Ok(logger) = ServerLogger::new(&data.server.name, data.server.port) {
        logger.get_log_file_size_bytes().unwrap_or(0)
    } else {
        0
    };

    let file_count = std::fs::read_dir(&server_dir)
        .map(|entries| entries.count())
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(json!({
        "server_id": data.server.id,
        "server_name": data.server.name,
        "port": data.server.port,
        "uptime_seconds": uptime,
        "status": "running",
        "hot_reload": {
            "enabled": true,
            "websocket_url": format!("ws://127.0.0.1:{}/ws/hot-reload", data.server.port),
            "watching_directory": server_dir,
            "file_watcher": "active"
        },
        "static_files": {
            "directory": server_dir,
            "file_count": file_count,
            "enabled": true,
            "template_based": true
        },
        "logging": {
            "file_size_bytes": log_file_size,
            "enabled": true
        },
        "endpoints_count": 10,
        "last_updated": uptime
    })))
}

pub async fn stats_handler(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let server_dir = format!("www/{}-[{}]", data.server.name, data.server.port);

    let stats = if let Ok(logger) = ServerLogger::new(&data.server.name, data.server.port) {
        logger.get_request_stats().await.unwrap_or_default()
    } else {
        Default::default()
    };

    Ok(HttpResponse::Ok().json(json!({
        "server_id": data.server.id,
        "server_name": data.server.name,
        "server_directory": server_dir,
        "total_requests": stats.total_requests,
        "unique_ips": stats.unique_ips,
        "error_requests": stats.error_requests,
        "security_alerts": stats.security_alerts,
        "performance_warnings": stats.performance_warnings,
        "avg_response_time_ms": stats.avg_response_time,
        "max_response_time_ms": stats.max_response_time,
        "total_bytes_sent": stats.total_bytes_sent,
        "uptime_seconds": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        "hot_reload_status": "active"
    })))
}

pub async fn health_handler(_data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": timestamp,
        "uptime": "running",
        "logging": "active",
        "static_files": "enabled",
        "template_system": "active",
        "hot_reload": "active",
        "file_watcher": "monitoring",
        "config": "loaded from TOML"
    })))
}

pub async fn ping_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "pong",
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        "server": "rush-sync-server",
        "message": "Ping received successfully"
    })))
}

// Static Message Store (In-Memory)
static MESSAGES: LazyLock<Arc<Mutex<VecDeque<Message>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(VecDeque::new())));
static MESSAGE_COUNTER: LazyLock<Arc<Mutex<u32>>> = LazyLock::new(|| Arc::new(Mutex::new(0)));

// POST /api/message - receive a message
pub async fn message_handler(body: web::Json<serde_json::Value>) -> ActixResult<HttpResponse> {
    let message_text = body
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("No message");

    let from = body
        .get("from")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let timestamp = body
        .get("timestamp")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| chrono::Local::now().to_rfc3339());

    // Store message
    let message_id = {
        let mut messages = MESSAGES.lock().unwrap_or_else(|p| p.into_inner());
        let mut counter = MESSAGE_COUNTER.lock().unwrap_or_else(|p| p.into_inner());
        *counter += 1;
        let id = *counter;

        messages.push_back(Message {
            message: message_text.to_string(),
            from: from.to_string(),
            timestamp: timestamp.to_string(),
            id,
        });

        // Keep at most 100 messages
        if messages.len() > 100 {
            messages.pop_front();
        }
        id
    };

    log::info!("Message received from {}: {}", from, message_text);

    Ok(HttpResponse::Ok().json(json!({
        "status": "received",
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        "message_id": message_id
    })))
}

// GET /api/messages - retrieve all messages
pub async fn messages_handler() -> ActixResult<HttpResponse> {
    let messages = {
        let messages_lock = MESSAGES.lock().unwrap_or_else(|p| p.into_inner());
        messages_lock.iter().cloned().collect::<Vec<_>>()
    };

    Ok(HttpResponse::Ok().json(json!({
        "messages": messages,
        "count": messages.len(),
        "status": "success"
    })))
}

pub async fn close_browser_handler() -> ActixResult<HttpResponse> {
    let html = r#"
<script>
setTimeout(() => { window.close(); }, 100);
document.write('<h1>Server stopped - closing...</h1>');
</script>
"#;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

// PUT /api/files/{path} — Upload/create a file
pub async fn upload_file(
    data: web::Data<ServerDataWithConfig>,
    path: web::Path<String>,
    body: web::Bytes,
) -> ActixResult<HttpResponse> {
    let file_path = path.into_inner();

    if file_path.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({"error": "Path required"})));
    }

    let base_dir = crate::core::helpers::get_base_dir().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Base dir error: {}", e))
    })?;
    let server_dir = base_dir
        .join("www")
        .join(format!("{}-[{}]", data.server.name, data.server.port));

    // Reject path traversal attempts early
    if file_path.contains("..") {
        return Ok(HttpResponse::Forbidden().json(json!({"error": "Path traversal blocked"})));
    }

    let target = server_dir.join(&file_path);

    // Create parent directories
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Directory creation failed: {}", e))
        })?;
    }

    // Path traversal protection (verify resolved path is within server dir)
    let canonical_server = server_dir
        .canonicalize()
        .unwrap_or_else(|_| server_dir.clone());
    if let Some(canonical_parent) = target.parent().and_then(|p| p.canonicalize().ok()) {
        if !canonical_parent.starts_with(&canonical_server) {
            return Ok(HttpResponse::Forbidden().json(json!({"error": "Path traversal blocked"})));
        }
    }

    let size = body.len();
    tokio::fs::write(&target, &body)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Write failed: {}", e)))?;

    log::info!(
        "File uploaded: {} ({} bytes) to {}-[{}]",
        file_path,
        size,
        data.server.name,
        data.server.port
    );

    Ok(HttpResponse::Ok().json(json!({
        "status": "uploaded",
        "path": file_path,
        "size": size
    })))
}

// GET /api/files — List files in server directory
pub async fn list_files(
    data: web::Data<ServerDataWithConfig>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    let base_dir = crate::core::helpers::get_base_dir().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Base dir error: {}", e))
    })?;
    let server_dir = base_dir
        .join("www")
        .join(format!("{}-[{}]", data.server.name, data.server.port));

    let subpath = query.get("path").map(|s| s.as_str()).unwrap_or("");

    if subpath.contains("..") {
        return Ok(HttpResponse::Forbidden().json(json!({"error": "Path traversal blocked"})));
    }

    let target = if subpath.is_empty() {
        server_dir.clone()
    } else {
        server_dir.join(subpath)
    };

    // Path traversal protection
    let canonical_server = server_dir
        .canonicalize()
        .unwrap_or_else(|_| server_dir.clone());
    if let Ok(canonical_target) = target.canonicalize() {
        if !canonical_target.starts_with(&canonical_server) {
            return Ok(HttpResponse::Forbidden().json(json!({"error": "Path traversal blocked"})));
        }
    }

    if !target.exists() {
        return Ok(HttpResponse::NotFound().json(json!({"error": "Directory not found"})));
    }

    let mut entries = vec![];
    if let Ok(mut dir) = tokio::fs::read_dir(&target).await {
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            let metadata = entry.metadata().await.ok();
            entries.push(json!({
                "name": name,
                "is_dir": metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0),
            }));
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "server_name": data.server.name,
        "port": data.server.port,
        "path": subpath,
        "files": entries,
        "count": entries.len()
    })))
}

// DELETE /api/files/{path} — Delete a file or directory
pub async fn delete_file(
    data: web::Data<ServerDataWithConfig>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let file_path = path.into_inner();

    if file_path.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({"error": "Path required"})));
    }

    if file_path.contains("..") {
        return Ok(HttpResponse::Forbidden().json(json!({"error": "Path traversal blocked"})));
    }

    let base_dir = crate::core::helpers::get_base_dir().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Base dir error: {}", e))
    })?;
    let server_dir = base_dir
        .join("www")
        .join(format!("{}-[{}]", data.server.name, data.server.port));

    let target = server_dir.join(&file_path);

    // Path traversal protection
    let canonical_server = server_dir
        .canonicalize()
        .unwrap_or_else(|_| server_dir.clone());
    if let Ok(canonical_target) = target.canonicalize() {
        if !canonical_target.starts_with(&canonical_server) {
            return Ok(HttpResponse::Forbidden().json(json!({"error": "Path traversal blocked"})));
        }
    }

    if !target.exists() {
        return Ok(HttpResponse::NotFound().json(json!({"error": "File not found"})));
    }

    if target.is_dir() {
        tokio::fs::remove_dir_all(&target).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Delete failed: {}", e))
        })?;
    } else {
        tokio::fs::remove_file(&target).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Delete failed: {}", e))
        })?;
    }

    log::info!(
        "File deleted: {} from {}-[{}]",
        file_path,
        data.server.name,
        data.server.port
    );

    Ok(HttpResponse::Ok().json(json!({
        "status": "deleted",
        "path": file_path
    })))
}

// ACME challenge handler for Let's Encrypt HTTP-01 validation
pub async fn acme_challenge_handler(path: web::Path<String>) -> ActixResult<HttpResponse> {
    let token = path.into_inner();
    if let Some(key_auth) = crate::server::acme::get_challenge_response(&token) {
        Ok(HttpResponse::Ok().content_type("text/plain").body(key_auth))
    } else {
        Ok(HttpResponse::NotFound().body("Challenge not found"))
    }
}

// GET /api/acme/status — ACME/TLS certificate status
pub async fn acme_status_handler() -> ActixResult<HttpResponse> {
    let status = crate::server::acme::get_acme_status();
    Ok(HttpResponse::Ok().json(status))
}

// GET /api/acme/dashboard — ACME/TLS status dashboard
pub async fn acme_dashboard_handler() -> ActixResult<HttpResponse> {
    let status = crate::server::acme::get_acme_status();
    let json_data = serde_json::to_string(&status)
        .unwrap_or_else(|_| "{}".to_string())
        .replace("</", "<\\/"); // Prevent XSS in inline script
    let html = crate::server::acme::ACME_DASHBOARD_HTML.replace("__ACME_DATA__", &json_data);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

// GET /api/analytics — Analytics summary JSON
pub async fn analytics_handler() -> ActixResult<HttpResponse> {
    let summary = crate::server::analytics::get_summary();
    Ok(HttpResponse::Ok().json(summary))
}

// GET /api/analytics/dashboard — Embedded analytics dashboard
pub async fn analytics_dashboard_handler() -> ActixResult<HttpResponse> {
    let summary = crate::server::analytics::get_summary();
    let json_data = serde_json::to_string(&summary)
        .unwrap_or_else(|_| "{}".to_string())
        .replace("</", "<\\/"); // Prevent XSS in inline script
    let html = crate::server::analytics::DASHBOARD_HTML.replace("__ANALYTICS_DATA__", &json_data);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
