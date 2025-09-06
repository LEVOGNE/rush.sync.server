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
    PERSISTENT_REGISTRY
        .get_or_init(|| ServerRegistry::new().expect("Failed to initialize server registry"))
}

pub fn get_proxy_manager() -> &'static Arc<ProxyManager> {
    PROXY_MANAGER.get_or_init(|| {
        // Config laden und Proxy Manager erstellen
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

// NEU: Proxy System starten
async fn start_proxy_system(config: &Config) -> crate::core::error::Result<()> {
    if !config.proxy.enabled {
        log::info!("Proxy system disabled in config");
        return Ok(());
    }

    let proxy_manager = get_proxy_manager();

    // Proxy Server starten (HTTP auf 8000 + HTTPS auf 8443)
    Arc::clone(proxy_manager).start_proxy_server().await?;

    log::info!("Proxy system started:");
    log::info!("  HTTP:  http://{{name}}.localhost:{}", config.proxy.port);

    let https_port = config.proxy.port + config.proxy.https_port_offset;
    log::info!("HTTPS: https://{{name}}.localhost:{}", https_port);

    Ok(())
}

// NEU: HTTP Redirect Server starten
// async fn start_http_redirect(config: &Config) -> crate::core::error::Result<()> {
//     if !config.proxy.enabled {
//         return Ok(());
//     }

//     // Port 80 nur wenn verfügbar
//     if !is_port_available(80) {
//         log::warn!("Port 80 already in use - HTTP redirect disabled");
//         log::info!("Tip: Use 'sudo lsof -i :80' to check what's using it");
//         return Ok(());
//     }

//     // Import aus server::redirect
//     use crate::server::redirect::HttpRedirectServer;

//     let redirect = HttpRedirectServer::new(80, 8443); // Redirect zu HTTPS Proxy

//     std::thread::spawn(move || {
//         // Single-thread Tokio-Runtime (keine Send-Anforderung für Futures)
//         let rt = tokio::runtime::Builder::new_current_thread()
//             .enable_all()
//             .build()
//             .expect("failed to build single-thread runtime");

//         rt.block_on(async move {
//             if let Err(e) = redirect.run().await {
//                 log::error!("HTTP redirect server failed: {}", e);
//             }
//         });
//     });

//     log::info!("HTTP→HTTPS redirect active on port 80");
//     Ok(())
// }

pub async fn initialize_server_system() -> crate::core::error::Result<()> {
    let config = Config::load().await?;

    crate::server::handlers::web::set_global_config(config.clone());

    let registry = get_persistent_registry();
    let context = get_shared_context();

    let mut persistent_servers = registry.load_servers().await?;
    let mut corrected_servers = 0;

    for (_server_id, persistent_info) in persistent_servers.iter_mut() {
        if persistent_info.status == ServerStatus::Running {
            if !is_port_available(persistent_info.port) {
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
        let mut servers = context.servers.write().unwrap();
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

    // ÜBERARBEITET: Proxy Manager mit verbesserter Struktur
    if config.proxy.enabled {
        // 1. Proxy System starten (HTTP + HTTPS)
        if let Err(e) = start_proxy_system(&config).await {
            log::error!("Failed to start proxy system: {}", e);
            // Proxy ist optional - wir laufen trotzdem weiter
        } else {
            // 2. HTTP Redirect starten (optional, braucht sudo für Port 80)
            if let Err(e) = start_http_redirect_server(&config).await {
                log::warn!("Failed to start HTTP redirect: {}", e);
                // Nicht fatal - läuft auch ohne
            }

            // 3. Bereits laufende Server beim Proxy registrieren
            let proxy_manager = get_proxy_manager();
            for (_id, persistent_info) in persistent_servers.iter() {
                if persistent_info.status == ServerStatus::Running {
                    if let Err(e) = proxy_manager
                        .add_route(
                            &persistent_info.name,
                            &persistent_info.id,
                            persistent_info.port,
                        )
                        .await
                    {
                        log::error!(
                            "Failed to register server {} with proxy: {}",
                            persistent_info.name,
                            e
                        );
                    } else {
                        log::info!(
                            "Registered existing server {} with proxy",
                            persistent_info.name
                        );
                    }
                }
            }
        }

        if is_port_available(80) {
            log::info!("  With sudo: http://{{name}}.localhost → redirects to HTTPS");
        }
        log::info!("  Add to /etc/hosts: 127.0.0.1 {{name}}.localhost");
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
        let mut handles = context.handles.write().unwrap();
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

        // Korrigierte API-Aufrufe
        let _ = registry
            .update_server_status(&server_id, ServerStatus::Stopped)
            .await;
    }

    log::info!("Server system shutdown complete");
    Ok(())
}

pub async fn validate_server_creation(
    name: &str,
    port: Option<u16>,
) -> crate::core::error::Result<()> {
    let config = Config::load().await?;
    let context = get_shared_context();
    let servers = context.servers.read().unwrap();

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
    let servers = context.servers.read().unwrap();

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
            "https_port": 8443,
            "redirect_port": if is_port_available(80) { None } else { Some(80) }
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
    let auto_start_servers = {
        let servers = registry.load_servers().await?;
        registry.get_auto_start_servers(&servers)
    };

    if auto_start_servers.is_empty() {
        return Ok(vec![]);
    }

    let max_to_start = config.server.max_concurrent.min(auto_start_servers.len());
    let mut started_servers = Vec::new();

    for server in auto_start_servers.iter().take(max_to_start) {
        log::info!(
            "Auto-starting server: {} on port {}",
            server.name,
            server.port
        );
        started_servers.push(format!("{}:{}", server.name, server.port));
    }

    if auto_start_servers.len() > max_to_start {
        log::warn!(
            "Skipped {} auto-start servers due to max_concurrent limit of {}",
            auto_start_servers.len() - max_to_start,
            config.server.max_concurrent
        );
    }

    Ok(started_servers)
}

// Füge das ganz am Ende der Datei ein, nach der letzten Funktion
// Ersetze die start_http_redirect_server Funktion in src/server/shared.rs

async fn start_http_redirect_server(_config: &Config) -> crate::core::error::Result<()> {
    let redirect_port = 80;
    let target_https_port = 8443;

    if !crate::server::utils::port::is_port_available(redirect_port) {
        log::warn!(
            "Port {} already in use - HTTP redirect disabled",
            redirect_port
        );
        return Ok(());
    }

    log::info!(
        "Starting HTTP→HTTPS redirect server on port {}",
        redirect_port
    );

    // LÖSUNG: std::thread::spawn statt tokio::spawn verwenden
    std::thread::spawn(move || {
        // Single-thread Tokio-Runtime (keine Send-Anforderung für Futures)
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

    // Kurz warten für Startup
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    log::info!(
        "HTTP redirect active: Port {} → HTTPS Port {}",
        redirect_port,
        target_https_port
    );

    Ok(())
}
