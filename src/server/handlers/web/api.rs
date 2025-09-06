use super::ServerDataWithConfig;
use crate::server::{config, logging::ServerLogger};
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
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
        "proxy_port": data.proxy_https_port, // Verwende https proxy port
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
        "proxy_port": data.proxy_https_port, // Verwende https proxy port
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
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        "server": "rush-sync-server",
        "message": "Ping received successfully"
    })))
}

// Static Message Store (In-Memory für Demo)
lazy_static::lazy_static! {
    static ref MESSAGES: Arc<Mutex<VecDeque<Message>>> = Arc::new(Mutex::new(VecDeque::new()));
    static ref MESSAGE_COUNTER: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
}

// POST /api/message - Nachricht empfangen
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

    // Message speichern
    {
        let mut messages = MESSAGES.lock().unwrap();
        let mut counter = MESSAGE_COUNTER.lock().unwrap();
        *counter += 1;

        let message = Message {
            message: message_text.to_string(),
            from: from.to_string(),
            timestamp: timestamp.to_string(),
            id: *counter,
        };

        messages.push_back(message);

        // Max 100 Messages behalten
        if messages.len() > 100 {
            messages.pop_front();
        }
    }

    // Message ID für Response merken
    let message_id = {
        let counter = MESSAGE_COUNTER.lock().unwrap();
        *counter
    };

    log::info!("Message received from {}: {}", from, message_text);

    Ok(HttpResponse::Ok().json(json!({
        "status": "received",
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        "message_id": message_id
    })))
}

// GET /api/messages - Alle Nachrichten abrufen
pub async fn messages_handler() -> ActixResult<HttpResponse> {
    let messages = {
        let messages_lock = MESSAGES.lock().unwrap();
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
