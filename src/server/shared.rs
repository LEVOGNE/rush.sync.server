use crate::core::config::Config;
use crate::proxy::ProxyManager;
use crate::server::persistence::ServerRegistry;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::port::is_port_available;
use std::sync::{Arc, OnceLock};

static SHARED_CONTEXT: OnceLock<ServerContext> = OnceLock::new();
static PERSISTENT_REGISTRY: OnceLock<ServerRegistry> = OnceLock::new();
static PROXY_MANAGER: OnceLock<Arc<ProxyManager>> = OnceLock::new();

pub fn get_shared_context() -> &'static ServerContext {
    SHARED_CONTEXT.get_or_init(ServerContext::default)
}

pub fn get_persistent_registry() -> &'static ServerRegistry {
    PERSISTENT_REGISTRY.get_or_init(|| match ServerRegistry::new() {
        Ok(registry) => registry,
        Err(e) => {
            log::error!(
                "Failed to initialize server registry: {}, using fallback",
                e
            );
            ServerRegistry::with_fallback()
        }
    })
}

pub fn get_proxy_manager() -> &'static Arc<ProxyManager> {
    PROXY_MANAGER.get_or_init(|| {
        // Load config and create proxy manager
        let config = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                crate::core::config::Config::load()
                    .await
                    .unwrap_or_default()
            })
        });

        Arc::new(ProxyManager::new(config.proxy))
    })
}

// Start the proxy system
async fn start_proxy_system(config: &Config) -> crate::core::error::Result<()> {
    if !config.proxy.enabled {
        log::info!("Proxy system disabled in config");
        return Ok(());
    }

    let proxy_manager = get_proxy_manager();

    // Start proxy server (HTTP + HTTPS)
    Arc::clone(proxy_manager).start_proxy_server().await?;

    log::info!("Proxy system started:");
    log::info!(
        "  HTTP:  http://{{name}}.{}:{}",
        config.server.production_domain,
        config.proxy.port
    );

    let https_port = config.proxy.port + config.proxy.https_port_offset;
    log::info!(
        "  HTTPS: https://{{name}}.{}:{}",
        config.server.production_domain,
        https_port
    );

    Ok(())
}

pub async fn initialize_server_system() -> crate::core::error::Result<()> {
    let config = Config::load().await?;

    crate::server::handlers::web::set_global_config(config.clone());

    let registry = get_persistent_registry();
    let context = get_shared_context();

    let mut persistent_servers = registry.load_servers().await?;
    let mut corrected_servers = 0;

    for (_server_id, persistent_info) in persistent_servers.iter_mut() {
        if persistent_info.status == ServerStatus::Running {
            if !is_port_available(persistent_info.port, &config.server.bind_address) {
                log::warn!(
                    "Server {} claims to be running on port {}, but port is occupied",
                    persistent_info.name,
                    persistent_info.port
                );
                persistent_info.status = ServerStatus::Failed;
                corrected_servers += 1;
            } else {
                log::info!(
                    "Server {} was running but is no longer active, correcting status",
                    persistent_info.name
                );
                persistent_info.status = ServerStatus::Stopped;
                corrected_servers += 1;
            }
        }
    }

    if corrected_servers > 0 {
        registry.save_servers(&persistent_servers).await?;
        log::info!(
            "Corrected {} server statuses after program restart",
            corrected_servers
        );
    }

    {
        let mut servers = context.servers.write().map_err(|e| {
            crate::core::error::AppError::Validation(format!("servers lock poisoned: {}", e))
        })?;
        servers.clear();
        for (id, persistent_info) in persistent_servers.iter() {
            let server_info = crate::server::types::ServerInfo::from(persistent_info.clone());
            servers.insert(id.clone(), server_info);
        }
    }

    log::info!(
        "Server system initialized with {} persistent servers",
        persistent_servers.len()
    );
    log::info!(
        "Server Config: Port Range {}-{}, Max Concurrent: {}, Workers: {}",
        config.server.port_range_start,
        config.server.port_range_end,
        config.server.max_concurrent,
        config.server.workers
    );
    log::info!(
        "Logging Config: Max Size {}MB, Archives: {}, Compression: {}, Request Logging: {}",
        config.logging.max_file_size_mb,
        config.logging.max_archive_files,
        config.logging.compress_archives,
        config.logging.log_requests
    );

    let auto_start_servers = registry.get_auto_start_servers(&persistent_servers);
    if !auto_start_servers.is_empty() {
        log::info!(
            "Found {} servers marked for auto-start",
            auto_start_servers.len()
        );

        if auto_start_servers.len() > config.server.max_concurrent {
            log::warn!(
                "Auto-start servers ({}) exceed max_concurrent ({}), some will be skipped",
                auto_start_servers.len(),
                config.server.max_concurrent
            );
        }
    }

    // Initialize proxy manager
    if config.proxy.enabled {
        // 1. Start proxy system (HTTP + HTTPS)
        if let Err(e) = start_proxy_system(&config).await {
            log::error!("Failed to start proxy system: {}", e);
        } else {
            // 2. Start HTTP redirect (port 80, serves ACME challenges + redirects to HTTPS)
            if let Err(e) = start_http_redirect_server(&config).await {
                log::warn!("Failed to start HTTP redirect: {}", e);
            }

            // Note: Proxy routes for auto-started servers are registered later
            // by create_web_server() in auto_start_servers(), not here.
        }

        log::info!(
            "  DNS: Point *.{} to this server",
            config.server.production_domain
        );

        // 4. Start ACME background provisioning + auto hot-reload
        // ACME runs AFTER proxy is ready (5s delay), provisions cert, then hot-reloads proxy TLS.
        // No manual restart needed — new connections automatically use the LE certificate.
        if config.server.use_lets_encrypt && config.server.production_domain != "localhost" {
            let cert_dir = crate::core::helpers::get_base_dir()
                .map(|b| b.join(&config.server.cert_dir))
                .unwrap_or_else(|_| std::path::PathBuf::from(&config.server.cert_dir));

            // Collect subdomains only from actually registered servers and proxy routes.
            // Every SAN must have a valid DNS record pointing to this server,
            // otherwise Let's Encrypt HTTP-01 validation fails for the entire certificate.
            // "blog" and "myapp" are built-in proxy routes (served directly, not via add_route).
            let mut subdomains: Vec<String> = vec!["blog".to_string(), "myapp".to_string()];
            for (_id, persistent_info) in persistent_servers.iter() {
                if !subdomains.contains(&persistent_info.name) {
                    subdomains.push(persistent_info.name.clone());
                }
            }
            let proxy_manager = get_proxy_manager();
            let routes = proxy_manager.get_routes().await;
            for route in &routes {
                if !subdomains.contains(&route.subdomain) {
                    subdomains.push(route.subdomain.clone());
                }
            }

            crate::server::acme::start_acme_background(
                config.server.production_domain.clone(),
                cert_dir,
                config.server.acme_email.clone(),
                false,
                subdomains,
            );
            log::info!(
                "ACME: Background provisioning + auto hot-reload started for {}",
                config.server.production_domain
            );
        }
    } else {
        log::info!("Reverse Proxy disabled in configuration");
    }

    Ok(())
}

pub async fn persist_server_update(server_id: &str, status: crate::server::types::ServerStatus) {
    let registry = get_persistent_registry();
    if let Err(e) = registry.update_server_status(server_id, status).await {
        log::error!("Failed to persist server status update: {}", e);
    }
}

pub async fn shutdown_all_servers_on_exit() -> crate::core::error::Result<()> {
    let config = Config::load().await.unwrap_or_default();
    let registry = get_persistent_registry();
    let context = get_shared_context();

    let server_handles: Vec<_> = {
        let mut handles = context.handles.write().map_err(|e| {
            crate::core::error::AppError::Validation(format!("handles lock poisoned: {}", e))
        })?;
        handles.drain().collect()
    };

    log::info!("Shutting down {} active servers...", server_handles.len());

    let shutdown_timeout = std::time::Duration::from_secs(config.server.shutdown_timeout);

    for (server_id, handle) in server_handles {
        log::info!("Stopping server {}", server_id);

        if tokio::time::timeout(shutdown_timeout, handle.stop(true))
            .await
            .is_err()
        {
            log::warn!("Server {} shutdown timeout, forcing stop", server_id);
            handle.stop(false).await;
        }

        // Persist stopped status
        let _ = registry
            .update_server_status(&server_id, ServerStatus::Stopped)
            .await;
    }

    // Save analytics data before exit
    crate::server::analytics::save_analytics_on_shutdown();

    log::info!("Server system shutdown complete");
    Ok(())
}

pub async fn validate_server_creation(
    name: &str,
    port: Option<u16>,
) -> crate::core::error::Result<()> {
    let config = Config::load().await?;
    let context = get_shared_context();
    let servers = context.servers.read().map_err(|e| {
        crate::core::error::AppError::Validation(format!("servers lock poisoned: {}", e))
    })?;

    if servers.len() >= config.server.max_concurrent {
        return Err(crate::core::error::AppError::Validation(format!(
            "Server limit reached: {}/{}. Use 'cleanup' command to remove stopped servers.",
            servers.len(),
            config.server.max_concurrent
        )));
    }

    if let Some(port) = port {
        if port < config.server.port_range_start || port > config.server.port_range_end {
            return Err(crate::core::error::AppError::Validation(format!(
                "Port {} outside configured range {}-{}",
                port, config.server.port_range_start, config.server.port_range_end
            )));
        }
    }

    if servers.values().any(|s| s.name == name) {
        return Err(crate::core::error::AppError::Validation(format!(
            "Server name '{}' already exists",
            name
        )));
    }

    Ok(())
}

pub async fn get_server_system_stats() -> serde_json::Value {
    let config = Config::load().await.unwrap_or_default();
    let context = get_shared_context();
    let servers = match context.servers.read() {
        Ok(s) => s,
        Err(_) => return serde_json::json!({"error": "lock poisoned"}),
    };

    let running_count = servers
        .values()
        .filter(|s| s.status == ServerStatus::Running)
        .count();
    let stopped_count = servers
        .values()
        .filter(|s| s.status == ServerStatus::Stopped)
        .count();
    let failed_count = servers
        .values()
        .filter(|s| s.status == ServerStatus::Failed)
        .count();

    serde_json::json!({
        "total_servers": servers.len(),
        "running": running_count,
        "stopped": stopped_count,
        "failed": failed_count,
        "max_concurrent": config.server.max_concurrent,
        "utilization_percent": (servers.len() as f64 / config.server.max_concurrent as f64 * 100.0),
        "port_range": format!("{}-{}", config.server.port_range_start, config.server.port_range_end),
        "available_ports": config.server.port_range_end - config.server.port_range_start + 1,
        "proxy": {
            "enabled": config.proxy.enabled,
            "http_port": config.proxy.port,
            "https_port": config.proxy.port + config.proxy.https_port_offset,
            "redirect_port": if is_port_available(80, "0.0.0.0") { None } else { Some(80) }
        },
        "config": {
            "workers_per_server": config.server.workers,
            "shutdown_timeout_sec": config.server.shutdown_timeout,
            "startup_delay_ms": config.server.startup_delay_ms,
            "logging": {
                "max_file_size_mb": config.logging.max_file_size_mb,
                "max_archives": config.logging.max_archive_files,
                "compression": config.logging.compress_archives,
                "request_logging": config.logging.log_requests,
                "security_alerts": config.logging.log_security_alerts,
                "performance_monitoring": config.logging.log_performance
            }
        }
    })
}

pub async fn auto_start_servers() -> crate::core::error::Result<Vec<String>> {
    let config = Config::load().await?;
    let registry = get_persistent_registry();
    let ctx = get_shared_context();
    let auto_start_list = {
        let servers = registry.load_servers().await?;
        registry.get_auto_start_servers(&servers)
    };

    if auto_start_list.is_empty() {
        return Ok(vec![]);
    }

    let max_to_start = config.server.max_concurrent.min(auto_start_list.len());
    let mut started_servers = Vec::new();

    for server in auto_start_list.iter().take(max_to_start) {
        log::info!(
            "Auto-starting server: {} on port {}",
            server.name,
            server.port
        );

        // Check port availability
        if !is_port_available(server.port, &config.server.bind_address) {
            log::warn!(
                "Port {} not available for server '{}', skipping",
                server.port,
                server.name
            );
            continue;
        }

        let server_info = crate::server::types::ServerInfo::from(server.clone());

        // Actually start the web server (bind port, serve HTTP)
        match crate::server::handlers::web::create_web_server(ctx, server_info.clone(), &config) {
            Ok(handle) => {
                // Store handle
                if let Ok(mut handles) = ctx.handles.write() {
                    handles.insert(server_info.id.clone(), handle);
                }

                // Update status in memory
                if let Ok(mut servers) = ctx.servers.write() {
                    if let Some(s) = servers.get_mut(&server_info.id) {
                        s.status = ServerStatus::Running;
                    }
                }

                // Persist status
                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    persist_server_update(&server_id, ServerStatus::Running).await;
                });

                started_servers.push(format!("{}:{}", server_info.name, server_info.port));
                log::info!(
                    "Server '{}' started on http://{}:{}",
                    server_info.name,
                    config.server.bind_address,
                    server_info.port
                );
            }
            Err(e) => {
                log::error!(
                    "Failed to auto-start server '{}': {}",
                    server.name,
                    e
                );

                // Mark as failed
                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    persist_server_update(&server_id, ServerStatus::Failed).await;
                });
            }
        }
    }

    if auto_start_list.len() > max_to_start {
        log::warn!(
            "Skipped {} auto-start servers due to max_concurrent limit of {}",
            auto_start_list.len() - max_to_start,
            config.server.max_concurrent
        );
    }

    Ok(started_servers)
}

async fn start_http_redirect_server(config: &Config) -> crate::core::error::Result<()> {
    let redirect_port = 80;
    // In Docker, host port 443 maps to container port 3443.
    // The redirect must use the EXTERNAL port that clients see (443), not the internal one (3443).
    // EXTERNAL_HTTPS_PORT env var overrides the computed internal port.
    let target_https_port = std::env::var("EXTERNAL_HTTPS_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or_else(|| config.proxy.port + config.proxy.https_port_offset);

    if !crate::server::utils::port::is_port_available(redirect_port, "0.0.0.0") {
        log::warn!(
            "Port {} already in use - HTTP redirect disabled",
            redirect_port
        );
        return Ok(());
    }

    log::info!(
        "Starting HTTP->HTTPS redirect server on port {}",
        redirect_port
    );

    // Use std::thread::spawn to avoid Send requirements on the future
    std::thread::spawn(move || {
        // Single-threaded tokio runtime for the redirect server
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build single-thread runtime for redirect server");

        rt.block_on(async move {
            let redirect_server =
                crate::server::redirect::HttpRedirectServer::new(redirect_port, target_https_port);

            if let Err(e) = redirect_server.run().await {
                log::error!("HTTP redirect server error: {}", e);
            }
        });
    });

    // Brief wait for startup
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    log::info!(
        "HTTP redirect active: Port {} → HTTPS Port {}",
        redirect_port,
        target_https_port
    );

    Ok(())
}
