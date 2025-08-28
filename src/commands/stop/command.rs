// Fixed src/commands/stop/command.rs
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::validation::find_server;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::timeout;

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

        // DON'T use block_on - instead use spawn_blocking for config loading
        let config_result = std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(Config::load())
        })
        .join()
        .map_err(|_| AppError::Validation("Failed to load config".to_string()))??;

        let ctx = crate::server::shared::get_shared_context();

        self.stop_server(&config_result, ctx, args[0])
    }

    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            if args.is_empty() {
                return Err(AppError::Validation(
                    "Server-ID/Name fehlt! Verwende 'stop <ID>'".to_string(),
                ));
            }

            let config = Config::load().await?;
            let ctx = crate::server::shared::get_shared_context();

            self.stop_server_async(&config, ctx, args[0]).await
        })
    }

    fn supports_async(&self) -> bool {
        true
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
        let server_info = {
            let servers = ctx.servers.read().unwrap();
            find_server(&servers, identifier)?.clone()
        };

        if server_info.status != ServerStatus::Running {
            return Ok(format!(
                "Server '{}' is not active (Status: {})",
                server_info.name, server_info.status
            ));
        }

        log::info!(
            "Stopping server {} on port {}",
            server_info.id,
            server_info.port
        );

        let handle_removed = {
            let mut handles = ctx.handles.write().unwrap();
            handles.remove(&server_info.id)
        };

        if let Some(handle) = handle_removed {
            self.shutdown_server_gracefully(handle, server_info.id.clone(), config);
            self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);

            // Persistence update
            let server_id = server_info.id.clone();
            tokio::spawn(async move {
                crate::server::shared::persist_server_update(&server_id, ServerStatus::Stopped)
                    .await;
            });

            // Use configurable startup delay for consistent timing
            std::thread::sleep(Duration::from_millis(config.server.startup_delay_ms));

            let running_count = {
                let servers = ctx.servers.read().unwrap();
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
            self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);

            // Persistence update
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

    async fn stop_server_async(
        &self,
        config: &Config,
        ctx: &ServerContext,
        identifier: &str,
    ) -> Result<String> {
        let server_info = {
            let servers = ctx.servers.read().unwrap();
            find_server(&servers, identifier)?.clone()
        };

        if server_info.status != ServerStatus::Running {
            return Ok(format!(
                "Server '{}' is not active (Status: {})",
                server_info.name, server_info.status
            ));
        }

        log::info!(
            "Stopping server {} on port {} (async)",
            server_info.id,
            server_info.port
        );

        let handle_removed = {
            let mut handles = ctx.handles.write().unwrap();
            handles.remove(&server_info.id)
        };

        if let Some(handle) = handle_removed {
            // Use configured shutdown timeout
            let shutdown_timeout_duration = Duration::from_secs(config.server.shutdown_timeout);

            match timeout(shutdown_timeout_duration, handle.stop(true)).await {
                Ok(_) => log::info!("Server {} stopped gracefully", server_info.id),
                Err(_) => {
                    log::warn!("Server {} shutdown timeout, forcing stop", server_info.id);
                    handle.stop(false).await;
                }
            }

            self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);

            // Persistence update
            let server_id = server_info.id.clone();
            tokio::spawn(async move {
                crate::server::shared::persist_server_update(&server_id, ServerStatus::Stopped)
                    .await;
            });

            let running_count = {
                let servers = ctx.servers.read().unwrap();
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
            self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);

            // Persistence update
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

    // Updated to use config for shutdown timeout
    fn shutdown_server_gracefully(
        &self,
        handle: actix_web::dev::ServerHandle,
        server_id: String,
        config: &Config,
    ) {
        let shutdown_timeout = config.server.shutdown_timeout;

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
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
