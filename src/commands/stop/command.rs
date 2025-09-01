// Fixed src/commands/stop/command.rs - MIT BROWSER CLOSE
use crate::commands::command::Command;
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
        "Stop a web server (persistent)"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("stop")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(AppError::Validation(
                "Server-ID/Name fehlt! Verwende 'stop <ID>'".to_string(),
            ));
        }

        let config = get_config()?;

        let ctx = crate::server::shared::get_shared_context();
        self.stop_server(&config, ctx, args[0])
    }

    fn priority(&self) -> u8 {
        67
    }
}

impl StopCommand {
    fn stop_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        identifier: &str,
    ) -> Result<String> {
        // Atomare Operationen für Race-Condition-Schutz
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

            // Handle atomisch entfernen
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

        // Status sofort auf "Stopping" setzen um Race Conditions zu vermeiden
        self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);

        // Browser-Benachrichtigung
        self.notify_browser_shutdown(&server_info);

        if let Some(handle) = handle {
            // Graceful shutdown
            self.shutdown_server_gracefully(handle, server_info.id.clone(), config);

            // Persistence update (nicht blockierend)
            let server_id = server_info.id.clone();
            tokio::spawn(async move {
                crate::server::shared::persist_server_update(&server_id, ServerStatus::Stopped)
                    .await;
            });

            // Kurze Pause für konsistente Timing
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
            // Handle war bereits weg - nur Status updaten
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

    fn notify_browser_shutdown(&self, server_info: &crate::server::types::ServerInfo) {
        let server_port = server_info.port;
        let server_name = server_info.name.clone();

        // Keine eigene Runtime: einfach task spawnen
        tokio::spawn(async move {
            let server_url = format!("http://127.0.0.1:{}", server_port);
            log::info!(
                "Notifying browser to close for server {} (async)",
                server_name
            );

            let client = reqwest::Client::new();
            if let Err(e) = client
                .get(format!("{}/api/close-browser", server_url)) // <- ohne &
                .timeout(std::time::Duration::from_millis(300))
                .send()
                .await
            {
                log::warn!("Failed to notify browser: {}", e);
            }

            // Mini-Pause, damit Browser reagieren kann
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        });
    }

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

    fn update_server_status(&self, ctx: &ServerContext, server_id: &str, status: ServerStatus) {
        if let Ok(mut servers) = ctx.servers.write() {
            if let Some(server) = servers.get_mut(server_id) {
                server.status = status;
            }
        }
    }
}
