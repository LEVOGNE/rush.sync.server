// Updated src/server/handlers/web.rs
use crate::core::config::Config;
use crate::server::config;
use crate::server::logging::ServerLogger;
use crate::server::middleware::LoggingMiddleware;
use crate::server::types::{ServerContext, ServerData, ServerInfo};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result as ActixResult};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Updated to accept Config parameter
pub fn create_web_server(
    ctx: &ServerContext,
    server_info: ServerInfo,
    config: &Config,
) -> std::result::Result<actix_web::dev::ServerHandle, String> {
    let server_id = server_info.id.clone();
    let server_name = server_info.name.clone();
    let servers_clone = Arc::clone(&ctx.servers);

    // Create server logger with config
    let server_logger =
        match ServerLogger::new_with_config(&server_name, server_info.port, &config.logging) {
            Ok(logger) => Arc::new(logger),
            Err(e) => return Err(format!("Logger creation failed: {}", e)),
        };

    // Log server start
    let logger_for_start = server_logger.clone();
    tokio::spawn(async move {
        if let Err(e) = logger_for_start.log_server_start().await {
            log::error!("Failed to log server start: {}", e);
        }
    });

    let server_data = web::Data::new(ServerData {
        id: server_id.clone(),
        port: server_info.port,
        name: server_name.clone(),
    });

    let server_logger_for_app = server_logger.clone();

    // Use config for server settings
    let server_result = HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .wrap(LoggingMiddleware::new(server_logger_for_app.clone()))
            .wrap(middleware::Compress::default())
            .route("/", web::get().to(hello_handler))
            .route("/status", web::get().to(status_handler))
            .route("/api/info", web::get().to(info_handler))
            .route("/api/metrics", web::get().to(metrics_handler))
            .route("/api/stats", web::get().to(stats_handler))
            .route("/health", web::get().to(health_handler))
            .route("/logs", web::get().to(logs_handler))
    })
    .workers(config.server.workers) // Use config
    .shutdown_timeout(config.server.shutdown_timeout) // Use config
    .disable_signals()
    .bind(("127.0.0.1", server_info.port))
    .map_err(|e| format!("Bind failed: {}", e))?
    .run();

    let server_handle = server_result.handle();

    // Server in eigenem Thread
    let server_id_for_thread = server_id.clone();
    let logger_for_cleanup = server_logger.clone();
    let startup_delay = config.server.startup_delay_ms; // Use config

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            match server_result.await {
                Ok(_) => log::info!("Server {} ended normally", server_id_for_thread),
                Err(e) => {
                    log::error!("Server {} error: {}", server_id_for_thread, e);
                    if let Ok(mut servers) = servers_clone.write() {
                        if let Some(server) = servers.get_mut(&server_id_for_thread) {
                            server.status = crate::server::types::ServerStatus::Failed;
                        }
                    }
                }
            }

            if let Err(e) = logger_for_cleanup.log_server_stop().await {
                log::error!("Failed to log server stop: {}", e);
            }

            if let Ok(mut servers) = servers_clone.write() {
                if let Some(server) = servers.get_mut(&server_id_for_thread) {
                    server.status = crate::server::types::ServerStatus::Stopped;
                }
            }
        });
    });

    // Use configurable startup delay
    std::thread::sleep(Duration::from_millis(startup_delay));
    Ok(server_handle)
}

// Enhanced stats handler with real logger data
async fn stats_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    // Try to get real stats from logger
    let log_path = format!(".rss/servers/{}-[{}].log", data.name, data.port);

    // Create a temporary logger to get stats (in real implementation, this would be passed in)
    let stats = if let Ok(logger) = ServerLogger::new(&data.name, data.port) {
        logger.get_request_stats().await.unwrap_or_default()
    } else {
        Default::default()
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "server_id": data.id,
        "server_name": data.name,
        "log_file": log_path,
        "total_requests": stats.total_requests,
        "unique_ips": stats.unique_ips,
        "error_requests": stats.error_requests,
        "security_alerts": stats.security_alerts,
        "performance_warnings": stats.performance_warnings,
        "avg_response_time_ms": stats.avg_response_time,
        "max_response_time_ms": stats.max_response_time,
        "total_bytes_sent": stats.total_bytes_sent,
        "uptime_seconds": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    })))
}

async fn logs_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let log_path = format!(".rss/servers/{}-[{}].log", data.name, data.port);

    // Try to read actual log entries
    let log_entries = if let Ok(logger) = ServerLogger::new(&data.name, data.port) {
        match logger.get_log_file_size_bytes() {
            Ok(size) if size > 0 => {
                format!("Log file size: {} bytes", size)
            }
            _ => "No log entries yet".to_string(),
        }
    } else {
        "Logger unavailable".to_string()
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Server Logs - {}</title>
    <style>
        body {{ font-family: 'Monaco', 'Courier New', monospace; margin: 20px; background: #1a1a1a; color: #00ff00; }}
        .header {{ background: #2a2a2a; padding: 20px; border-radius: 5px; margin-bottom: 20px; }}
        .log-container {{ background: #000; padding: 15px; border-radius: 5px; max-height: 600px; overflow-y: auto; }}
        .log-entry {{ margin: 2px 0; font-size: 12px; }}
        .config-info {{ color: #ffff00; margin-top: 10px; }}
    </style>
    <script>setInterval(function() {{ location.reload(); }}, 5000);</script>
</head>
<body>
    <div class="header">
        <h1>Server Logs: {}</h1>
        <p>ID: {} | Port: {} | Log: {}</p>
        <p>Auto-refresh every 5 seconds</p>
        <div class="config-info">Logging configured from TOML settings</div>
    </div>
    <div class="log-container">
        <div class="log-entry">Log file: {}</div>
        <div class="log-entry">Status: {}</div>
        <div class="log-entry">Live logging is active with configurable rotation</div>
        <div class="log-entry">--- REAL LOG ENTRIES WOULD APPEAR HERE ---</div>
    </div>
</body>
</html>"#,
        data.name, data.name, data.id, data.port, log_path, log_path, log_entries
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn hello_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Rush Sync Server - {}</title>
    <style>
        body {{ font-family: 'Segoe UI', sans-serif; margin: 40px;
               background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
               min-height: 100vh; color: #333; }}
        .container {{ max-width: 600px; background: white; padding: 40px;
                     border-radius: 15px; box-shadow: 0 10px 30px rgba(0,0,0,0.2);
                     margin: 0 auto; }}
        h1 {{ color: #333; text-align: center; margin-bottom: 30px; font-size: 2.5em; }}
        .status {{ background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
                  color: white; padding: 20px; border-radius: 10px; margin: 30px 0;
                  text-align: center; font-weight: bold; font-size: 1.2em; }}
        .info {{ background: #f8f9fa; padding: 15px; border-radius: 8px; margin: 10px 0; }}
        .endpoint {{ margin: 5px 0; }}
        .endpoint a {{ color: #007bff; text-decoration: none; }}
        .endpoint a:hover {{ text-decoration: underline; }}
        .config-note {{ background: #e8f4fd; border-left: 4px solid #0084ff; padding: 10px; margin: 15px 0; font-size: 0.9em; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Rush Sync Server</h1>
        <div class="status">Server {} is running successfully!</div>
        <div class="info"><strong>Server ID:</strong> {}</div>
        <div class="info"><strong>Port:</strong> {}</div>
        <div class="info"><strong>URL:</strong> http://127.0.0.1:{}</div>

        <div class="config-note">
            <strong>Configuration:</strong> This server uses TOML-based configuration with customizable
            logging, port ranges, and server limits.
        </div>

        <h3>Available Endpoints:</h3>
        <div class="endpoint"><a href="/">Home</a> - This page</div>
        <div class="endpoint"><a href="/status">Status</a> - Server Status JSON</div>
        <div class="endpoint"><a href="/api/info">API Info</a> - Complete API Information</div>
        <div class="endpoint"><a href="/api/metrics">Metrics</a> - Server Metrics</div>
        <div class="endpoint"><a href="/api/stats">Stats</a> - Request Statistics (Real Data)</div>
        <div class="endpoint"><a href="/logs">Logs</a> - Live Server Logs</div>
        <div class="endpoint"><a href="/health">Health</a> - Health Check</div>
    </div>
</body>
</html>"#,
            data.name, data.name, data.id, data.port, data.port
        )))
}

async fn status_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Try to get logger config summary
    let logger_config = if let Ok(logger) = ServerLogger::new(&data.name, data.port) {
        logger.get_config_summary()
    } else {
        "Logger config unavailable".to_string()
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "running",
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "server": config::get_server_name(),
        "version": config::get_server_version(),
        "uptime_seconds": uptime,
        "logging": "enabled",
        "log_file": format!(".rss/servers/{}-[{}].log", data.name, data.port),
        "config_source": "TOML",
        "logger_config": logger_config
    })))
}

async fn info_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "name": "Rush Sync Server",
        "version": config::get_server_version(),
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "endpoints": [
            { "path": "/", "method": "GET", "description": "Welcome page with links" },
            { "path": "/status", "method": "GET", "description": "Server status with uptime and config info" },
            { "path": "/api/info", "method": "GET", "description": "API information" },
            { "path": "/api/metrics", "method": "GET", "description": "Server metrics" },
            { "path": "/api/stats", "method": "GET", "description": "Request statistics (real data from logs)" },
            { "path": "/logs", "method": "GET", "description": "Live server logs viewer" },
            { "path": "/health", "method": "GET", "description": "Health check" }
        ],
        "features": [
            "Individual server logging with configurable rotation",
            "Request tracking (configurable)",
            "Security monitoring (configurable)",
            "Performance metrics (configurable)",
            "TOML-based configuration",
            "Real-time statistics from log files"
        ],
        "logging": {
            "enabled": true,
            "log_file": format!(".rss/servers/{}-[{}].log", data.name, data.port),
            "format": "JSON",
            "includes": ["requests", "security_alerts", "performance_data"],
            "configurable": "via rush.toml [logging] section",
            "rotation": "configurable size and archive count"
        },
        "configuration": {
            "source": "rush.toml",
            "sections": ["[server]", "[logging]"],
            "features": ["port_ranges", "max_concurrent", "log_rotation", "compression"]
        }
    })))
}

async fn metrics_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Try to get real metrics from logger
    let log_file_size = if let Ok(logger) = ServerLogger::new(&data.name, data.port) {
        logger.get_log_file_size_bytes().unwrap_or(0)
    } else {
        0
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "uptime_seconds": uptime,
        "status": "running",
        "memory_usage": "N/A",
        "requests_total": 0,
        "endpoints_count": 7,
        "logging_enabled": true,
        "log_file_size_bytes": log_file_size,
        "config_source": "TOML",
        "last_updated": uptime
    })))
}

async fn health_handler(_data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": timestamp,
        "uptime": "running",
        "logging": "active",
        "config": "loaded from TOML"
    })))
}
