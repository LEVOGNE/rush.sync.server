// src/server/handlers/web.rs - CLEANED VERSION
use crate::server::types::{ServerContext, ServerData, ServerInfo};
use actix_web::{web, App, HttpResponse, HttpServer, Result as ActixResult};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn create_web_server(
    ctx: &ServerContext,
    server_info: ServerInfo,
) -> std::result::Result<actix_web::dev::ServerHandle, String> {
    let server_id = server_info.id.clone();
    let server_name = server_info.name.clone();
    let servers_clone = Arc::clone(&ctx.servers);

    // Server-Daten für Handler
    let server_data = web::Data::new(ServerData {
        id: server_id.clone(),
        port: server_info.port,
        name: server_name.clone(),
    });

    // Server mit korrekter Konfiguration
    let server_result = HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .route("/", web::get().to(hello_handler))
            .route("/status", web::get().to(status_handler))
            .route("/api/info", web::get().to(info_handler))
            .route("/api/metrics", web::get().to(metrics_handler)) // ✅ Added useful metrics
            .route("/health", web::get().to(health_handler))
    })
    .workers(1) // Einzelner Worker für embedded use
    .shutdown_timeout(5) // Schneller Shutdown
    .disable_signals() // Manuelle Signal-Kontrolle
    .bind(("127.0.0.1", server_info.port))
    .map_err(|e| format!("Bind failed: {}", e))?
    .run();

    let server_handle = server_result.handle();

    // Server in eigenem Thread mit proper cleanup
    let server_id_for_thread = server_id.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            // ✅ SIMPLIFIED: Removed shutdown_token complexity
            match server_result.await {
                Ok(_) => log::info!("🌐 Server {} ended normally", server_id_for_thread),
                Err(e) => {
                    log::error!("❌ Server {} error: {}", server_id_for_thread, e);
                    // Status auf Failed setzen
                    if let Ok(mut servers) = servers_clone.write() {
                        if let Some(server) = servers.get_mut(&server_id_for_thread) {
                            server.status = crate::server::types::ServerStatus::Failed;
                        }
                    }
                }
            }

            // Status auf Stopped setzen nach Beendigung
            if let Ok(mut servers) = servers_clone.write() {
                if let Some(server) = servers.get_mut(&server_id_for_thread) {
                    server.status = crate::server::types::ServerStatus::Stopped;
                    log::info!("📊 Server {} status set to STOPPED", server_id_for_thread);
                }
            }
        });
    });

    // Kurz warten um sicherzustellen, dass Server gestartet ist
    std::thread::sleep(Duration::from_millis(500));

    Ok(server_handle)
}

async fn hello_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="de">
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
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Rush Sync Server</h1>
        <div class="status">✅ Server {} läuft erfolgreich!</div>
        <div class="info"><strong>Server ID:</strong> {}</div>
        <div class="info"><strong>Port:</strong> {}</div>
        <div class="info"><strong>URL:</strong> http://127.0.0.1:{}</div>

        <h3>📍 Available Endpoints:</h3>
        <div class="endpoint">🏠 <a href="/">Home</a> - Diese Seite</div>
        <div class="endpoint">📊 <a href="/status">Status</a> - Server Status JSON</div>
        <div class="endpoint">📋 <a href="/api/info">API Info</a> - Complete API Information</div>
        <div class="endpoint">📈 <a href="/api/metrics">Metrics</a> - Server Metrics</div>
        <div class="endpoint">💚 <a href="/health">Health</a> - Health Check</div>
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

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "running",
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "server": "Rush Sync Server",
        "version": "1.0.0",
        "uptime_seconds": uptime
    })))
}

async fn info_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "name": "Rush Sync Server",
        "version": "1.0.0",
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "endpoints": [
            { "path": "/", "method": "GET", "description": "Welcome page with links" },
            { "path": "/status", "method": "GET", "description": "Server status with uptime" },
            { "path": "/api/info", "method": "GET", "description": "API information" },
            { "path": "/api/metrics", "method": "GET", "description": "Server metrics" },
            { "path": "/health", "method": "GET", "description": "Health check" }
        ],
        "created_with": "Rust + Actix-Web",
        "workers": 1
    })))
}

// ✅ MERGED: Combined functionality from api.rs
async fn metrics_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "uptime_seconds": uptime,
        "status": "running",
        "memory_usage": "N/A", // Could be implemented with sysinfo crate
        "requests_total": 0,    // Could be tracked with Arc<AtomicU64>
        "endpoints_count": 5,
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
        "uptime": "running"
    })))
}
