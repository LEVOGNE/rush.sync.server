// ===== src/server/handlers/web/mod.rs =====
pub mod api;
pub mod assets;
pub mod logs;
pub mod server;
pub mod templates;

// Re-exports für einfache Verwendung
pub use api::*;
pub use assets::*;
pub use logs::*;
pub use server::*;
pub use templates::*;

use crate::core::config::Config;
use crate::server::logging::ServerLogger;
use crate::server::middleware::LoggingMiddleware;
use crate::server::tls::TlsManager;
use crate::server::types::{ServerContext, ServerData, ServerInfo};
use crate::server::watchdog::{get_watchdog_manager, ws_hot_reload};
use actix_web::{middleware, web, App, HttpServer};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// Konstante für Proxy-Port
pub const PROXY_PORT: u16 = 8443;

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

    // Template-Pfade korrigiert für neue Struktur
    let readme_template = include_str!("../templates/README.md");
    let readme_content = readme_template
        .replace("{{SERVER_NAME}}", server_name)
        .replace("{{PORT}}", &port.to_string());
    std::fs::write(server_dir.join("README.md"), readme_content)
        .map_err(crate::core::error::AppError::Io)?;

    let robots_template = include_str!("../templates/robots.txt");
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
    let watchdog_manager = get_watchdog_manager().clone();

    let tls_config = if config.server.enable_https && config.server.auto_cert {
        match TlsManager::new(&config.server.cert_dir, config.server.cert_validity_days) {
            Ok(tls_manager) => match tls_manager.get_rustls_config(&server_name, server_port) {
                Ok(rustls_config) => {
                    log::info!("TLS certificate loaded for {}:{}", server_name, server_port);
                    Some(rustls_config)
                }
                Err(e) => {
                    log::error!("TLS setup failed: {}", e);
                    None
                }
            },
            Err(e) => {
                log::error!("TLS manager creation failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let mut http_server = HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .app_data(web::Data::new(watchdog_manager.clone()))
            .wrap(LoggingMiddleware::new(server_logger_for_app.clone()))
            .wrap(middleware::Compress::default())
            // Assets
            .route("/rss.js", web::get().to(serve_rss_js))
            .route("/.rss/", web::get().to(serve_system_dashboard))
            .route("/.rss/style.css", web::get().to(serve_system_css))
            .route("/.rss/favicon.svg", web::get().to(serve_system_favicon))
            .route("/.rss/fonts/{font}", web::get().to(serve_quicksand_font))
            .route(
                "/.rss/global-reset.css",
                web::get().to(serve_global_reset_css),
            )
            // API
            .route("/api/status", web::get().to(status_handler))
            .route("/api/health", web::get().to(health_handler))
            .route("/api/info", web::get().to(info_handler))
            .route("/api/metrics", web::get().to(metrics_handler))
            .route("/api/stats", web::get().to(stats_handler))
            .route("/api/close-browser", web::get().to(close_browser_handler))
            // Logs
            .route("/api/logs", web::get().to(logs_handler))
            .route("/api/logs/raw", web::get().to(logs_raw_handler))
            // WebSocket
            .route("/ws/hot-reload", web::get().to(ws_hot_reload))
            // Fallback
            .default_service(web::route().to(serve_fallback_or_inject))
    })
    .workers(config.server.workers)
    .shutdown_timeout(config.server.shutdown_timeout)
    .disable_signals();

    http_server = http_server
        .bind(("127.0.0.1", server_info.port))
        .map_err(|e| format!("HTTP bind failed: {}", e))?;

    if tls_config.is_some() {
        let https_port = server_port + config.server.https_port_offset;
        log::info!("TLS certificate ready for HTTPS on port {}", https_port);
        log::info!(
            "Certificate: .rss/certs/{}-{}.cert",
            server_name,
            server_port
        );
    }

    let server_result = http_server.run();
    let server_handle = server_result.handle();

    let server_id_for_thread = server_id.clone();
    let logger_for_cleanup = server_logger.clone();
    let startup_delay = config.server.startup_delay_ms;
    let server_name_for_cleanup = server_name.clone();
    let server_port_for_cleanup = server_port;

    if config.proxy.enabled {
        let proxy_manager = crate::server::shared::get_proxy_manager();
        let proxy_server_name = server_name.clone();
        let proxy_server_id = server_id.clone();
        let proxy_server_port = server_port;
        let startup_delay_clone = startup_delay;

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                startup_delay_clone + 100,
            ))
            .await;

            if let Err(e) = proxy_manager
                .add_route(&proxy_server_name, &proxy_server_id, proxy_server_port)
                .await
            {
                log::error!(
                    "Failed to register server {} with proxy: {}",
                    proxy_server_name,
                    e
                );
            } else {
                log::info!(
                    "Server {} registered with proxy: {}.localhost -> 127.0.0.1:{}",
                    proxy_server_name,
                    proxy_server_name,
                    proxy_server_port
                );
            }
        });
    }

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
