pub mod api;
pub mod assets;
pub mod logs;
pub mod server;
pub mod templates;

pub use api::*;
pub use assets::*;
pub use logs::*;
pub use server::*;
pub use templates::*;

use crate::core::config::Config;
use crate::server::logging::ServerLogger;
use crate::server::middleware::{ApiKeyAuth, LoggingMiddleware, RateLimiter};
use crate::server::tls::TlsManager;
use crate::server::types::{ServerContext, ServerData, ServerInfo};
use crate::server::watchdog::{get_watchdog_manager, ws_hot_reload};
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

// Set the global config (called once at startup)
pub fn set_global_config(config: Config) {
    let _ = GLOBAL_CONFIG.set(config);
}

pub fn get_proxy_http_port() -> u16 {
    // HTTP proxy runs on the configured proxy port (default 3000)
    GLOBAL_CONFIG.get().map(|c| c.proxy.port).unwrap_or(3000)
}

pub fn get_proxy_https_port() -> u16 {
    // HTTPS proxy runs on HTTP port + https_port_offset
    GLOBAL_CONFIG
        .get()
        .map(|c| c.proxy.port + c.proxy.https_port_offset)
        .unwrap_or(3443)
}

pub fn create_server_directory_and_files(
    server_name: &str,
    port: u16,
) -> crate::core::error::Result<PathBuf> {
    let base_dir = crate::core::helpers::get_base_dir()?;

    let server_dir = base_dir
        .join("www")
        .join(format!("{}-[{}]", server_name, port));
    std::fs::create_dir_all(&server_dir).map_err(crate::core::error::AppError::Io)?;

    // Generate files from templates
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

    // Build server data with proxy port configuration
    let server_data = web::Data::new(ServerDataWithConfig {
        server: ServerData {
            id: server_id.clone(),
            port: server_info.port,
            name: server_name.clone(),
        },
        proxy_http_port: get_proxy_http_port(),
        proxy_https_port: get_proxy_https_port(),
    });

    let server_logger_for_app = server_logger.clone();
    let watchdog_manager = get_watchdog_manager().clone();

    let tls_config = if config.server.enable_https && config.server.auto_cert {
        match TlsManager::new(&config.server.cert_dir, config.server.cert_validity_days) {
            Ok(tls_manager) => match tls_manager.get_rustls_config_for_domain(
                &server_name,
                server_port,
                &config.server.production_domain,
            ) {
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

    let production_domain = config.server.production_domain.clone();
    let api_key = config.server.api_key.clone();
    let rate_limit_rps = config.server.rate_limit_rps;
    let rate_limit_enabled = config.server.rate_limit_enabled;
    let mut http_server = HttpServer::new(move || {
        let prod_domain = production_domain.clone();
        App::new()
            .app_data(server_data.clone())
            .app_data(web::Data::new(watchdog_manager.clone()))
            .wrap(LoggingMiddleware::new(server_logger_for_app.clone()))
            .wrap(RateLimiter::new(rate_limit_rps, rate_limit_enabled))
            .wrap(ApiKeyAuth::new(api_key.clone()))
            .wrap(middleware::Compress::default())
            .wrap(
                Cors::default()
                    .allowed_origin_fn(move |origin, _req_head| {
                        let origin_str = origin.to_str().unwrap_or("");
                        // Always allow local development
                        let is_local =
                            origin_str.contains("127.0.0.1") || origin_str.contains("localhost");
                        if is_local {
                            return true;
                        }
                        // Allow production domain if configured
                        if prod_domain != "localhost" {
                            return origin_str.contains(&prod_domain);
                        }
                        false
                    })
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            // Assets
            .route("/.rss/_reset.css", web::get().to(serve_global_reset_css))
            .route("/.rss/style.css", web::get().to(serve_system_css))
            .route("/.rss/favicon.svg", web::get().to(serve_system_favicon))
            .route("/.rss/", web::get().to(serve_system_dashboard))
            // Font Assets
            .route("/.rss/fonts/{font}", web::get().to(serve_quicksand_font))
            // JavaScript Assets
            .route("/rss.js", web::get().to(serve_rss_js))
            .route("/.rss/js/rush-app.js", web::get().to(serve_rush_app_js))
            .route("/.rss/js/rush-api.js", web::get().to(serve_rush_api_js))
            .route("/.rss/js/rush-ui.js", web::get().to(serve_rush_ui_js))
            // API Routes (specific before generic)
            .route("/api/status", web::get().to(status_handler))
            .route("/api/health", web::get().to(health_handler))
            .route("/api/info", web::get().to(info_handler))
            .route("/api/metrics", web::get().to(metrics_handler))
            .route("/api/stats", web::get().to(stats_handler))
            .route("/api/ping", web::post().to(ping_handler))
            .route("/api/message", web::post().to(message_handler))
            .route("/api/messages", web::get().to(messages_handler))
            .route("/api/close-browser", web::get().to(close_browser_handler))
            .route("/api/logs", web::get().to(logs_handler))
            .route("/api/logs/raw", web::get().to(logs_raw_handler))
            .route("/api/acme/status", web::get().to(acme_status_handler))
            .route("/api/acme/dashboard", web::get().to(acme_dashboard_handler))
            .route("/api/analytics", web::get().to(analytics_handler))
            .route("/api/analytics/dashboard", web::get().to(analytics_dashboard_handler))
            // File Management API
            .route("/api/files", web::get().to(list_files))
            .route("/api/files/{path:.*}", web::put().to(upload_file))
            .route("/api/files/{path:.*}", web::delete().to(delete_file))
            // ACME Challenge (Let's Encrypt)
            .route(
                "/.well-known/acme-challenge/{token}",
                web::get().to(acme_challenge_handler),
            )
            // WebSocket Routes
            .route("/ws/hot-reload", web::get().to(ws_hot_reload))
            // Fallback (must be last)
            .default_service(web::route().to(serve_fallback_or_inject))
    })
    .workers(config.server.workers)
    .shutdown_timeout(config.server.shutdown_timeout)
    .disable_signals();

    http_server = http_server
        .bind((&*config.server.bind_address, server_info.port))
        .map_err(|e| format!("HTTP bind failed: {}", e))?;

    if let Some(tls_cfg) = tls_config {
        let https_port = server_port + config.server.https_port_offset;
        let bind_result = http_server.bind_rustls_021(
            (&*config.server.bind_address, https_port),
            tls_cfg.as_ref().clone(),
        );
        match bind_result {
            Ok(server) => {
                http_server = server;
                log::info!("HTTPS active for {} on port {}", server_name, https_port);
            }
            Err(e) => {
                log::error!(
                    "HTTPS bind failed for {} on port {}: {}",
                    server_name,
                    https_port,
                    e
                );
                log::info!("Continuing with HTTP only");
                // http_server was consumed by bind_rustls_021, need to return error
                return Err(format!("HTTPS bind failed: {}", e));
            }
        }
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
        let bind_addr = config.server.bind_address.clone();

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
                    "Server {} registered with proxy: {} -> {}:{}",
                    proxy_server_name,
                    proxy_server_name,
                    bind_addr,
                    proxy_server_port
                );
            }
        });
    }

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                log::error!(
                    "Failed to create runtime for server {}: {}",
                    server_id_for_thread,
                    e
                );
                if let Ok(mut servers) = servers_clone.write() {
                    if let Some(server) = servers.get_mut(&server_id_for_thread) {
                        server.status = crate::server::types::ServerStatus::Failed;
                    }
                }
                return;
            }
        };
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

#[derive(Debug, Clone)]
pub struct ServerDataWithConfig {
    pub server: ServerData,
    pub proxy_http_port: u16,
    pub proxy_https_port: u16,
}
