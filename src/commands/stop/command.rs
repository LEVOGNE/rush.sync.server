use crate::commands::command::Command;
use crate::commands::parsing::{parse_bulk_args, BulkMode};
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::validation::find_server;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct StopCommand;

impl StopCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for StopCommand {
    fn name(&self) -> &'static str {
        "stop"
    }

    fn description(&self) -> &'static str {
        "Stop server(s) - supports ranges and bulk operations"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("stop")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(AppError::Validation(get_translation(
                "server.error.id_missing",
                &[],
            )));
        }

        let config = get_config()?;
        let ctx = crate::server::shared::get_shared_context();

        match parse_bulk_args(args) {
            BulkMode::Single(identifier) => self.stop_single_server(&config, ctx, &identifier),
            BulkMode::Range(start, end) => self.stop_range_servers(&config, ctx, start, end),
            BulkMode::All => self.stop_all_servers(&config, ctx),
            BulkMode::Invalid(error) => Err(AppError::Validation(error)),
        }
    }

    fn priority(&self) -> u8 {
        67
    }
}

impl StopCommand {
    // Stop single server
    fn stop_single_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        identifier: &str,
    ) -> Result<String> {
        let (server_info, handle) = {
            let servers_guard = ctx
                .servers
                .read()
                .map_err(|_| AppError::Validation("Server-Context lock poisoned".to_string()))?;

            let server_info = find_server(&servers_guard, identifier)?.clone();

            if server_info.status != ServerStatus::Running {
                return Ok(format!(
                    "Server '{}' is not active (Status: {})",
                    server_info.name, server_info.status
                ));
            }

            // Atomically remove the handle
            let handle = {
                let mut handles_guard = ctx.handles.write().map_err(|_| {
                    AppError::Validation("Handle-Context lock poisoned".to_string())
                })?;
                handles_guard.remove(&server_info.id)
            };

            (server_info, handle)
        };

        log::info!(
            "Stopping server {} on port {}",
            server_info.id,
            server_info.port
        );

        // Set status to Stopped immediately
        self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);

        // Notify browser to close
        self.notify_browser_shutdown(&server_info);

        if let Some(handle) = handle {
            // Graceful shutdown
            self.shutdown_server_gracefully(handle, server_info.id.clone(), config);

            // Persist status update (non-blocking)
            let server_id = server_info.id.clone();
            tokio::spawn(async move {
                crate::server::shared::persist_server_update(&server_id, ServerStatus::Stopped)
                    .await;
            });

            // Brief pause for consistent timing
            std::thread::sleep(Duration::from_millis(
                config.server.startup_delay_ms.min(500),
            ));

            let running_count = {
                let servers = ctx.servers.read().unwrap_or_else(|e| {
                    log::warn!("Server lock poisoned: {}", e);
                    e.into_inner()
                });
                servers
                    .values()
                    .filter(|s| s.status == ServerStatus::Running)
                    .count()
            };

            Ok(format!(
                "Server '{}' stopped [PERSISTENT] ({}/{} running)",
                server_info.name, running_count, config.server.max_concurrent
            ))
        } else {
            // Handle was already removed - just update status
            let server_id = server_info.id.clone();
            tokio::spawn(async move {
                crate::server::shared::persist_server_update(&server_id, ServerStatus::Stopped)
                    .await;
            });

            Ok(format!(
                "Server '{}' was already stopped [PERSISTENT]",
                server_info.name
            ))
        }
    }

    // Stop servers by range (e.g., "stop 1-3")
    fn stop_range_servers(
        &self,
        config: &Config,
        ctx: &ServerContext,
        start: u32,
        end: u32,
    ) -> Result<String> {
        let mut results = Vec::new();
        let mut stopped_count = 0;
        let mut failed_count = 0;

        for i in start..=end {
            let identifier = format!("{}", i);

            match self.stop_single_server(config, ctx, &identifier) {
                Ok(message) => {
                    if message.contains("stopped [PERSISTENT]") {
                        stopped_count += 1;
                        results.push(format!("Server {}: Stopped", i));
                    } else {
                        results.push(format!("Server {}: {}", i, message));
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    results.push(format!("Server {}: Failed - {}", i, e));
                }
            }
        }

        let summary = format!(
            "Range stop completed: {} stopped, {} failed (Range: {}-{})",
            stopped_count, failed_count, start, end
        );

        if results.is_empty() {
            Ok(summary)
        } else {
            Ok(format!("{}\n\nResults:\n{}", summary, results.join("\n")))
        }
    }

    // Stop all running servers
    fn stop_all_servers(&self, config: &Config, ctx: &ServerContext) -> Result<String> {
        let running_servers: Vec<_> = {
            let servers = read_lock(&ctx.servers, "servers")?;
            servers
                .values()
                .filter(|s| s.status == ServerStatus::Running)
                .map(|s| (s.id.clone(), s.name.clone()))
                .collect()
        };

        if running_servers.is_empty() {
            return Ok("No running servers to stop".to_string());
        }

        if running_servers.len() > 20 {
            return Err(AppError::Validation(
                "Too many servers to stop at once (max 20). Use ranges instead.".to_string(),
            ));
        }

        let mut results = Vec::new();
        let mut stopped_count = 0;
        let mut failed_count = 0;

        // Stop servers in parallel for better performance
        let server_stops: Vec<_> = running_servers
            .into_iter()
            .map(|(server_id, server_name)| {
                match self.stop_single_server(config, ctx, &server_id) {
                    Ok(message) => {
                        if message.contains("stopped [PERSISTENT]") {
                            stopped_count += 1;
                            (server_name, "Stopped".to_string())
                        } else {
                            (server_name, message)
                        }
                    }
                    Err(e) => {
                        failed_count += 1;
                        (server_name, format!("Failed - {}", e))
                    }
                }
            })
            .collect();

        for (server_name, result) in server_stops {
            results.push(format!("{}: {}", server_name, result));
        }

        let summary = format!(
            "Stop all completed: {} stopped, {} failed",
            stopped_count, failed_count
        );

        Ok(format!("{}\n\nResults:\n{}", summary, results.join("\n")))
    }

    // Browser notification
    fn notify_browser_shutdown(&self, server_info: &crate::server::types::ServerInfo) {
        let server_port = server_info.port;
        let server_name = server_info.name.clone();

        tokio::spawn(async move {
            let server_url = format!("http://127.0.0.1:{}", server_port);
            log::info!(
                "Notifying browser to close for server {} (async)",
                server_name
            );

            let client = reqwest::Client::new();
            if let Err(e) = client
                .get(format!("{}/api/close-browser", server_url))
                .timeout(std::time::Duration::from_millis(300))
                .send()
                .await
            {
                log::warn!("Failed to notify browser: {}", e);
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        });
    }

    // Graceful shutdown
    fn shutdown_server_gracefully(
        &self,
        handle: actix_web::dev::ServerHandle,
        server_id: String,
        config: &Config,
    ) {
        let shutdown_timeout = config.server.shutdown_timeout;

        tokio::spawn(async move {
            use tokio::time::{timeout, Duration};

            match timeout(Duration::from_secs(shutdown_timeout), handle.stop(true)).await {
                Ok(_) => log::info!("Server {} stopped gracefully", server_id),
                Err(_) => {
                    log::warn!(
                        "Server {} shutdown timeout ({}s), forcing stop",
                        server_id,
                        shutdown_timeout
                    );
                    handle.stop(false).await;
                }
            }
        });
    }

    // Status update helper
    fn update_server_status(&self, ctx: &ServerContext, server_id: &str, status: ServerStatus) {
        if let Ok(mut servers) = ctx.servers.write() {
            if let Some(server) = servers.get_mut(server_id) {
                server.status = status;
            }
        }
    }
}
