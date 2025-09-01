use crate::core::config::Config;
use crate::server::persistence::ServerRegistry;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::port::is_port_available;
use std::sync::OnceLock;

static SHARED_CONTEXT: OnceLock<ServerContext> = OnceLock::new();
static PERSISTENT_REGISTRY: OnceLock<ServerRegistry> = OnceLock::new();

pub fn get_shared_context() -> &'static ServerContext {
    SHARED_CONTEXT.get_or_init(ServerContext::default)
}

pub fn get_persistent_registry() -> &'static ServerRegistry {
    PERSISTENT_REGISTRY
        .get_or_init(|| ServerRegistry::new().expect("Failed to initialize server registry"))
}

pub async fn initialize_server_system() -> crate::core::error::Result<()> {
    let config = Config::load().await?;
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

    Ok(())
}

pub async fn persist_server_update(server_id: &str, status: crate::server::types::ServerStatus) {
    let registry = get_persistent_registry();
    if let Ok(servers) = registry.load_servers().await {
        if let Err(e) = registry
            .update_server_status(servers, server_id, status)
            .await
        {
            log::error!("Failed to persist server status update: {}", e);
        }
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

        if let Ok(servers) = registry.load_servers().await {
            let _ = registry
                .update_server_status(servers, &server_id, ServerStatus::Stopped)
                .await;
        }
    }

    if let Ok(mut servers) = registry.load_servers().await {
        let mut updated = false;
        for server in servers.values_mut() {
            if server.status == ServerStatus::Running {
                server.status = ServerStatus::Stopped;
                updated = true;
            }
        }

        if updated {
            let _ = registry.save_servers(&servers).await;
            log::info!("Updated all running servers to stopped status");
        }
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
