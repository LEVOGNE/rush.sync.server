use crate::core::config::Config;
use crate::server::config;
use crate::server::logging::ServerLogger;
use crate::server::middleware::LoggingMiddleware;
use crate::server::types::{ServerContext, ServerData, ServerInfo};
use crate::server::watchdog::{get_watchdog_manager, ws_hot_reload};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result as ActixResult};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn create_server_directory_and_files(
    server_name: &str,
    port: u16,
) -> crate::core::error::Result<PathBuf> {
    let exe_path = std::env::current_exe().map_err(crate::core::error::AppError::Io)?;
    let base_dir = exe_path.parent().ok_or_else(|| {
        crate::core::error::AppError::Validation(
            "Cannot determine executable directory".to_string(),
        )
    })?;

    let server_dir = base_dir
        .join("www")
        .join(format!("{}-[{}]", server_name, port));
    std::fs::create_dir_all(&server_dir).map_err(crate::core::error::AppError::Io)?;

    // README aus Template mit Variablen-Ersetzung
    let readme_template = include_str!("templates/README.md");
    let readme_content = readme_template
        .replace("{{SERVER_NAME}}", server_name)
        .replace("{{PORT}}", &port.to_string());
    std::fs::write(server_dir.join("README.md"), readme_content)
        .map_err(crate::core::error::AppError::Io)?;

    // robots.txt aus Template
    let robots_template = include_str!("templates/robots.txt");
    let robots_content = robots_template.replace("{{PORT}}", &port.to_string());
    std::fs::write(server_dir.join("robots.txt"), robots_content)
        .map_err(crate::core::error::AppError::Io)?;

    log::info!("Created development directory: {:?}", server_dir);
    log::info!("Files created: README.md, robots.txt");
    Ok(server_dir)
}

pub fn create_web_server(
    ctx: &ServerContext,
    server_info: ServerInfo,
    config: &Config,
) -> std::result::Result<actix_web::dev::ServerHandle, String> {
    let server_id = server_info.id.clone();
    let server_name = server_info.name.clone();
    let server_port = server_info.port;
    let servers_clone = Arc::clone(&ctx.servers);

    let server_logger =
        match ServerLogger::new_with_config(&server_name, server_info.port, &config.logging) {
            Ok(logger) => Arc::new(logger),
            Err(e) => return Err(format!("Logger creation failed: {}", e)),
        };

    // File Watching starten
    if let Err(e) = crate::server::watchdog::start_server_watching(&server_name, server_port) {
        log::warn!("Failed to start file watching for {}: {}", server_name, e);
    } else {
        log::info!(
            "File watching started for server {} on port {}",
            server_name,
            server_port
        );
    }

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

    // Watchdog Manager für WebSocket
    let watchdog_manager = get_watchdog_manager().clone();

    let server_result = HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .app_data(web::Data::new(watchdog_manager.clone()))
            .wrap(LoggingMiddleware::new(server_logger_for_app.clone()))
            .wrap(middleware::Compress::default())
            // 1. Hot-Reload Script (am wichtigsten)
            .route("/rss.js", web::get().to(serve_rss_js))
            // 2. System-Dashboard und Assets
            .route("/.rss/", web::get().to(serve_system_dashboard))
            .route("/.rss/style.css", web::get().to(serve_system_css))
            .route("/.rss/favicon.svg", web::get().to(serve_system_favicon))
            // 3. API-Endpoints (häufig verwendet)
            .route("/api/status", web::get().to(status_handler))
            .route("/api/health", web::get().to(health_handler))
            .route("/api/info", web::get().to(info_handler))
            .route("/api/metrics", web::get().to(metrics_handler))
            .route("/api/stats", web::get().to(stats_handler))
            .route("/api/logs", web::get().to(logs_handler))
            .route("/api/close-browser", web::get().to(close_browser_handler))
            // 4. WebSocket (weniger häufig)
            .route("/ws/hot-reload", web::get().to(ws_hot_reload))
            // 5. Fallback (muss IMMER zuletzt)
            .route("/.rss/fonts/{font}", web::get().to(serve_quicksand_font))
            .route(
                "/.rss/global-reset.css",
                web::get().to(serve_global_reset_css),
            )
            .default_service(web::route().to(serve_fallback_or_inject))
    })
    .workers(config.server.workers)
    .shutdown_timeout(config.server.shutdown_timeout)
    .disable_signals()
    .bind(("127.0.0.1", server_info.port))
    .map_err(|e| format!("Bind failed: {}", e))?
    .run();

    let server_handle = server_result.handle();

    let server_id_for_thread = server_id.clone();
    let logger_for_cleanup = server_logger.clone();
    let startup_delay = config.server.startup_delay_ms;
    let server_name_for_cleanup = server_name.clone();
    let server_port_for_cleanup = server_port;

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

            // File Watching stoppen
            if let Err(e) = crate::server::watchdog::stop_server_watching(
                &server_name_for_cleanup,
                server_port_for_cleanup,
            ) {
                log::warn!("Failed to stop file watching: {}", e);
            } else {
                log::info!(
                    "File watching stopped for server {}",
                    server_name_for_cleanup
                );
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

    std::thread::sleep(Duration::from_millis(startup_delay));
    Ok(server_handle)
}

async fn serve_system_dashboard(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let template = include_str!("templates/rss/dashboard.html");
    let html_content = template
        .replace("{{SERVER_NAME}}", &data.name)
        .replace("{{PORT}}", &data.port.to_string())
        .replace("{{VERSION}}", crate::server::config::get_server_version())
        .replace("{{CREATION_TIME}}", &chrono::Local::now().to_rfc3339());

    let html_with_script = inject_rss_script(html_content);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_with_script))
}

async fn serve_rss_js(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let js_content = include_str!("templates/rss/rss.js")
        .replace("{{SERVER_NAME}}", &data.name)
        .replace("{{PORT}}", &data.port.to_string());

    Ok(HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache"))
        .body(js_content))
}

async fn serve_system_css() -> ActixResult<HttpResponse> {
    let css_content = include_str!("templates/rss/style.css");

    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache"))
        .body(css_content))
}

async fn serve_system_favicon() -> ActixResult<HttpResponse> {
    let favicon_content = include_str!("templates/rss/favicon.svg");

    Ok(HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(favicon_content))
}

async fn serve_quicksand_font(req: actix_web::HttpRequest) -> ActixResult<HttpResponse> {
    let path = req.match_info().get("font").unwrap_or("Quicksand_Book.otf");

    // Validierung der Font-Datei
    let valid_fonts = [
        "Kenyan_Coffee_Bd_It.otf",
        "Kenyan_Coffee_Bd.otf",
        "Kenyan_Coffee_Rg_It.otf",
        "Kenyan_Coffee_Rg.otf",
    ];

    if !valid_fonts.contains(&path) {
        return Ok(HttpResponse::NotFound().body("Font not found"));
    }

    // Font-Datei laden mit Slice-Cast
    let font_data: &[u8] = match path {
        "Kenyan_Coffee_Bd_It.otf" => {
            include_bytes!("templates/rss/fonts/Kenyan_Coffee_Bd_It.otf").as_slice()
        }
        "Kenyan_Coffee_Bd.otf" => {
            include_bytes!("templates/rss/fonts/Kenyan_Coffee_Bd.otf").as_slice()
        }
        "Kenyan_Coffee_Rg_It.otf" => {
            include_bytes!("templates/rss/fonts/Kenyan_Coffee_Rg_It.otf").as_slice()
        }
        "Kenyan_Coffee_Rg.otf" => {
            include_bytes!("templates/rss/fonts/Kenyan_Coffee_Rg.otf").as_slice()
        }
        _ => return Ok(HttpResponse::NotFound().body("Font not found")),
    };

    Ok(HttpResponse::Ok()
        .content_type("font/otf")
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .body(font_data))
}

async fn serve_global_reset_css() -> ActixResult<HttpResponse> {
    let reset_css = include_str!("templates/rss/global-reset.css");

    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .insert_header(("Cache-Control", "public, max-age=3600"))
        .body(reset_css))
}

async fn serve_fallback_or_inject(
    req: actix_web::HttpRequest,
    data: web::Data<ServerData>,
) -> ActixResult<HttpResponse> {
    let path = req.path();
    log::info!("Requested path: {}", path);

    let exe_path = std::env::current_exe().unwrap();
    let base_dir = exe_path.parent().unwrap();
    let server_dir = base_dir
        .join("www")
        .join(format!("{}-[{}]", data.name, data.port));

    let file_path = if path == "/" {
        server_dir.join("index.html")
    } else {
        server_dir.join(path.trim_start_matches('/'))
    };

    log::info!("Looking for file: {:?}", file_path);
    log::info!("File exists: {}", file_path.exists());

    if file_path.exists() {
        if let Some(extension) = file_path.extension() {
            if extension == "html" {
                log::info!("Loading custom HTML file");
                match tokio::fs::read_to_string(&file_path).await {
                    Ok(mut html_content) => {
                        log::info!("Original HTML length: {}", html_content.len());
                        log::info!(
                            "Contains /rss.js already: {}",
                            html_content.contains("/rss.js")
                        );

                        if !html_content.contains("/rss.js") {
                            html_content = inject_rss_script(html_content);
                            log::info!("Injected script, new length: {}", html_content.len());
                        }

                        return Ok(HttpResponse::Ok()
                            .content_type("text/html; charset=utf-8")
                            .body(html_content));
                    }
                    Err(e) => {
                        log::error!("Failed to read HTML file: {}", e);
                    }
                }
            } else {
                log::info!("Serving static file: {:?}", file_path);
                match tokio::fs::read(&file_path).await {
                    Ok(content) => {
                        let content_type = match extension.to_str() {
                            Some("css") => "text/css",
                            Some("js") => "application/javascript",
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("svg") => "image/svg+xml",
                            _ => "application/octet-stream",
                        };

                        return Ok(HttpResponse::Ok().content_type(content_type).body(content));
                    }
                    Err(e) => {
                        log::error!("Failed to read file: {}", e);
                    }
                }
            }
        }
    }

    // Fallback nur für root path
    if path == "/" {
        log::info!("Serving system fallback");
        serve_system_fallback(data).await
    } else {
        log::info!("File not found: {}", path);
        Ok(HttpResponse::NotFound()
            .content_type("text/plain")
            .body("File not found"))
    }
}

fn inject_rss_script(html: String) -> String {
    let script_tag = r#"<script src="/rss.js"></script>"#;
    let css_link = r#"<link rel="stylesheet" href="/.rss/global-reset.css">"#;

    // Zuerst CSS in <head> einbetten
    let html_with_css = if let Some(head_end) = html.find("</head>") {
        let (before, after) = html.split_at(head_end);
        format!("{}\n    {}\n{}", before, css_link, after)
    } else {
        // Falls kein <head> existiert, am Anfang hinzufügen
        format!("{}\n{}", css_link, html)
    };

    // Dann Script wie gewohnt
    if let Some(body_end) = html_with_css.rfind("</body>") {
        let (before, after) = html_with_css.split_at(body_end);
        format!("{}\n    {}\n{}", before, script_tag, after)
    } else if let Some(html_end) = html_with_css.rfind("</html>") {
        let (before, after) = html_with_css.split_at(html_end);
        format!("{}\n{}\n{}", before, script_tag, after)
    } else {
        format!("{}\n{}", html_with_css, script_tag)
    }
}

async fn serve_system_fallback(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let template = include_str!("templates/rss/dashboard.html");
    let html_content = template
        .replace("{{SERVER_NAME}}", &data.name)
        .replace("{{PORT}}", &data.port.to_string())
        .replace("{{VERSION}}", crate::server::config::get_server_version())
        .replace("{{CREATION_TIME}}", &chrono::Local::now().to_rfc3339());

    let html_with_script = inject_rss_script(html_content);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_with_script))
}

async fn status_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let server_dir = format!("www/{}-[{}]", data.name, data.port);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "running",
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "server": config::get_server_name(),
        "version": config::get_server_version(),
        "uptime_seconds": uptime,
        "static_files": true,
        "template_system": true,
        "hot_reload": true,
        "websocket_endpoint": "/ws/hot-reload",
        "server_directory": server_dir,
        "log_file": format!(".rss/servers/{}-[{}].log", data.name, data.port)
    })))
}

async fn info_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let server_dir = format!("www/{}-[{}]", data.name, data.port);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "name": "Rush Sync Server",
        "version": config::get_server_version(),
        "server_id": data.id,
        "server_name": data.name,
        "port": data.port,
        "static_files_enabled": true,
        "template_system": "enabled",
        "hot_reload_enabled": true,
        "websocket_url": format!("ws://127.0.0.1:{}/ws/hot-reload", data.port),
        "server_directory": server_dir,
        "endpoints": [
            { "path": "/", "method": "GET", "description": "Static files from server directory", "type": "static" },
            { "path": "/favicon.svg", "method": "GET", "description": "SVG favicon", "type": "static" },
            { "path": "/api/status", "method": "GET", "description": "Server status", "type": "api" },
            { "path": "/api/info", "method": "GET", "description": "API information", "type": "api" },
            { "path": "/api/metrics", "method": "GET", "description": "Server metrics", "type": "api" },
            { "path": "/api/stats", "method": "GET", "description": "Request statistics", "type": "api" },
            { "path": "/api/logs", "method": "GET", "description": "Live server logs", "type": "api" },
            { "path": "/api/health", "method": "GET", "description": "Health check", "type": "api" },
            { "path": "/ws/hot-reload", "method": "GET", "description": "WebSocket hot reload", "type": "websocket" }
        ]
    })))
}

async fn metrics_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
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

    Ok(HttpResponse::Ok().json(serde_json::json!({
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
        "endpoints_count": 9,
        "last_updated": uptime
    })))
}

async fn stats_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let server_dir = format!("www/{}-[{}]", data.name, data.port);

    let stats = if let Ok(logger) = ServerLogger::new(&data.name, data.port) {
        logger.get_request_stats().await.unwrap_or_default()
    } else {
        Default::default()
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
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

async fn logs_handler(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let server_dir = format!("www/{}-[{}]", data.name, data.port);
    let log_path = format!(".rss/servers/{}-[{}].log", data.name, data.port);

    let log_entries = if let Ok(logger) = ServerLogger::new(&data.name, data.port) {
        match logger.get_log_file_size_bytes() {
            Ok(size) if size > 0 => format!("Log file size: {} bytes", size),
            _ => "No log entries yet".to_string(),
        }
    } else {
        "Logger unavailable".to_string()
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="de">
<head>
   <meta charset="UTF-8">
   <meta name="viewport" content="width=device-width, initial-scale=1.0">
   <title>Server Logs - {}</title>
   <link rel="icon" href="/favicon.svg" type="image/svg+xml">
   <style>
       body {{
           font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', 'Monaco', monospace;
           margin: 0;
           background: #1a1d23;
           color: #ffffff;
           padding: 1rem;
       }}
       .header {{
           background: #252830;
           padding: 1rem;
           border-radius: 6px;
           border: 1px solid #3a3f47;
           margin-bottom: 1rem;
       }}
       .header h1 {{
           margin: 0 0 0.75rem 0;
           font-size: 18px;
           color: #ffffff;
       }}
       .server-info {{
           font-size: 11px;
           color: #a0a6b1;
           line-height: 1.5;
       }}
       .back-link {{
           color: #00d4ff;
           text-decoration: none;
           font-size: 11px;
       }}
       .back-link:hover {{
           background: #00d4ff;
           color: #1a1d23;
           padding: 2px 4px;
           border-radius: 3px;
       }}
       .hot-reload-status {{
           color: #00ff88;
           font-weight: bold;
       }}
       .log-container {{
           background: #0d1117;
           border: 1px solid #3a3f47;
           border-radius: 6px;
           padding: 1rem;
           max-height: 600px;
           overflow-y: auto;
       }}
       .log-entry {{
           margin: 2px 0;
           font-size: 11px;
           color: #58a6ff;
           line-height: 1.3;
       }}
   </style>
   <script>setInterval(function() {{ location.reload(); }}, 5000);</script>
</head>
<body>
   <div class="header">
       <h1>Server Logs: {}</h1>
       <div class="server-info">
           <p>ID: {} | Port: {} | Directory: {}</p>
           <p>Log: {} | Auto-refresh: 5s</p>
           <p class="hot-reload-status">Hot Reload: ACTIVE (WebSocket on /ws/hot-reload)</p>
           <p><a href="/" class="back-link">← Zurück zur Hauptseite</a></p>
       </div>
   </div>
   <div class="log-container">
       <div class="log-entry">Server Directory: {}</div>
       <div class="log-entry">Log Status: {}</div>
       <div class="log-entry">Static Files: Enabled (Template-based)</div>
       <div class="log-entry">Hot Reload: WebSocket active on /ws/hot-reload</div>
       <div class="log-entry">File Watcher: Monitoring www directory for changes</div>
       <div class="log-entry">Configuration: Loaded from rush.toml</div>
       <div class="log-entry">--- REAL LOG ENTRIES WOULD APPEAR HERE ---</div>
       <div class="log-entry">Live logging with rotation, security alerts, and performance monitoring</div>
   </div>
</body>
</html>"#,
        data.name, data.name, data.id, data.port, server_dir, log_path, server_dir, log_entries
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn close_browser_handler() -> ActixResult<HttpResponse> {
    let html = r#"
<script>
setTimeout(() => { window.close(); }, 100);
document.write('<h1>Server stopped - closing...</h1>');
</script>
"#;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
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
        "static_files": "enabled",
        "template_system": "active",
        "hot_reload": "active",
        "file_watcher": "monitoring",
        "config": "loaded from TOML"
    })))
}
