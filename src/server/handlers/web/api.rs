// ===== src/server/handlers/web/api.rs =====
use super::PROXY_PORT;
use crate::server::{config, logging::ServerLogger, types::ServerData};
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn status_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let server_dir = format!("www/{}-[{}]", data.name, data.port);

    Ok(HttpResponse::Ok().json(json!({
        "status": "running",
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "proxy_port": PROXY_PORT,
        "server": config::get_server_name(),
        "version": config::get_server_version(),
        "uptime_seconds": uptime,
        "static_files": true,
        "template_system": true,
        "hot_reload": true,
        "websocket_endpoint": "/ws/hot-reload",
        "server_directory": server_dir,
        "log_file": format!(".rss/servers/{}-[{}].log", data.name, data.port),
        "certificate_file": format!(".rss/certs/{}-{}.cert", data.name, data.port),
        "private_key_file": format!(".rss/certs/{}-{}.key", data.name, data.port),
        "urls": {
            "http": format!("http://127.0.0.1:{}", data.port),
            "proxy": format!("https://{}.localhost:{}", data.name, PROXY_PORT)
        }
    })))
}

pub async fn info_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let server_dir = format!("www/{}-[{}]", data.name, data.port);

    Ok(HttpResponse::Ok().json(json!({
        "name": "Rush Sync Server",
        "version": config::get_server_version(),
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "proxy_port": PROXY_PORT,
        "static_files_enabled": true,
        "template_system": "enabled",
        "hot_reload_enabled": true,
        "websocket_url": format!("ws://127.0.0.1:{}/ws/hot-reload", data.port),
        "server_directory": server_dir,
        "certificate": {
            "cert_file": format!(".rss/certs/{}-{}.cert", data.name, data.port),
            "key_file": format!(".rss/certs/{}-{}.key", data.name, data.port),
            "common_name": format!("{}.localhost", data.name)
        },
        "urls": {
            "http": format!("http://127.0.0.1:{}", data.port),
            "proxy": format!("https://{}.localhost:{}", data.name, PROXY_PORT),
            "websocket": format!("ws://127.0.0.1:{}/ws/hot-reload", data.port)
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

pub async fn metrics_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let server_dir = format!("www/{}-[{}]", data.name, data.port);
    let log_file_size = if let Ok(logger) = ServerLogger::new(&data.name, data.port) {
        logger.get_log_file_size_bytes().unwrap_or(0)
    } else {
        0
    };

    let file_count = std::fs::read_dir(&server_dir)
        .map(|entries| entries.count())
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(json!({
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "uptime_seconds": uptime,
        "status": "running",
        "hot_reload": {
            "enabled": true,
            "websocket_url": format!("ws://127.0.0.1:{}/ws/hot-reload", data.port),
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

pub async fn stats_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let server_dir = format!("www/{}-[{}]", data.name, data.port);

    let stats = if let Ok(logger) = ServerLogger::new(&data.name, data.port) {
        logger.get_request_stats().await.unwrap_or_default()
    } else {
        Default::default()
    };

    Ok(HttpResponse::Ok().json(json!({
        "server_id": data.id,
        "server_name": data.name,
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

pub async fn health_handler(_data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
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

pub async fn close_browser_handler() -> ActixResult<HttpResponse> {
    let html = r#"
<script>
setTimeout(() => { window.close(); }, 100);
document.write('<h1>Server stopped - closing...</h1>');
</script>
"#;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
