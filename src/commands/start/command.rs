// Fixed src/commands/start/command.rs
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::port::is_port_available;
use crate::server::utils::validation::find_server;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Default)]
pub struct StartCommand;

impl StartCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for StartCommand {
    fn name(&self) -> &'static str {
        "start"
    }
    fn description(&self) -> &'static str {
        "Start a web server (persistent)"
    }
    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("start")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(AppError::Validation(
                "Server-ID/Name fehlt! Verwende 'start <ID>'".to_string(),
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

        self.start_server_sync(&config_result, ctx, args[0])
    }

    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            if args.is_empty() {
                return Err(AppError::Validation(
                    "Server-ID/Name fehlt! Verwende 'start <ID>'".to_string(),
                ));
            }

            let config = Config::load().await?;
            let ctx = crate::server::shared::get_shared_context();

            self.start_server_async(&config, ctx, args[0]).await
        })
    }

    fn supports_async(&self) -> bool {
        true
    }
    fn priority(&self) -> u8 {
        66
    }
}

impl StartCommand {
    fn start_server_sync(
        &self,
        config: &Config,
        ctx: &ServerContext,
        identifier: &str,
    ) -> Result<String> {
        let server_info = {
            let servers = ctx.servers.read().unwrap();
            find_server(&servers, identifier)?.clone()
        };

        self.validate_and_start_server(config, ctx, server_info)
    }

    async fn start_server_async(
        &self,
        config: &Config,
        ctx: &ServerContext,
        identifier: &str,
    ) -> Result<String> {
        let server_info = {
            let servers = ctx.servers.read().unwrap();
            find_server(&servers, identifier)?.clone()
        };

        self.validate_and_start_server(config, ctx, server_info)
    }

    fn validate_and_start_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        server_info: crate::server::types::ServerInfo,
    ) -> Result<String> {
        // Enhanced status validation
        if server_info.status == ServerStatus::Running {
            if !is_port_available(server_info.port) {
                // Port is occupied but not by us - correct status
                self.update_server_status(ctx, &server_info.id, ServerStatus::Failed);

                // Persistence update
                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    crate::server::shared::persist_server_update(&server_id, ServerStatus::Failed)
                        .await;
                });

                return Ok(format!(
                    "Port {} is occupied by another process! Server status corrected to FAILED. Use different port.",
                    server_info.port
                ));
            } else {
                // Port is free but status is Running -> correct to Stopped
                self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);

                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    crate::server::shared::persist_server_update(&server_id, ServerStatus::Stopped)
                        .await;
                });

                log::info!(
                    "Corrected server status from Running to Stopped for {}",
                    server_info.name
                );
            }
        }

        // Check port availability
        if !is_port_available(server_info.port) {
            return Ok(format!("Port {} already occupied!", server_info.port));
        }

        // Check server limits
        let running_servers = {
            let servers = ctx.servers.read().unwrap();
            servers
                .values()
                .filter(|s| s.status == ServerStatus::Running)
                .count()
        };

        if running_servers >= config.server.max_concurrent {
            return Err(AppError::Validation(format!(
                "Cannot start server: Running servers limit reached ({}/{}). Stop other servers first or increase max_concurrent in config.",
                running_servers, config.server.max_concurrent
            )));
        }

        // Validate port is within configured range
        if server_info.port < config.server.port_range_start
            || server_info.port > config.server.port_range_end
        {
            log::warn!(
                "Server {} port {} is outside configured range {}-{}, but starting anyway",
                server_info.name,
                server_info.port,
                config.server.port_range_start,
                config.server.port_range_end
            );
        }

        // Start the server
        match self.spawn_server(config, ctx, server_info.clone()) {
            Ok(handle) => {
                {
                    let mut handles = ctx.handles.write().unwrap();
                    handles.insert(server_info.id.clone(), handle);
                }
                self.update_server_status(ctx, &server_info.id, ServerStatus::Running);

                // Persistence update
                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    crate::server::shared::persist_server_update(&server_id, ServerStatus::Running)
                        .await;
                });

                Ok(format!(
                    "Server '{}' successfully started on http://127.0.0.1:{} [PERSISTENT] ({}/{} running)",
                    server_info.name,
                    server_info.port,
                    running_servers + 1,
                    config.server.max_concurrent
                ))
            }
            Err(e) => {
                self.update_server_status(ctx, &server_info.id, ServerStatus::Failed);

                // Persistence update
                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    crate::server::shared::persist_server_update(&server_id, ServerStatus::Failed)
                        .await;
                });

                Err(AppError::Validation(format!("Server start failed: {}", e)))
            }
        }
    }

    // Updated to use config
    fn spawn_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        server_info: crate::server::types::ServerInfo,
    ) -> std::result::Result<actix_web::dev::ServerHandle, String> {
        // Pass config to create_web_server
        crate::server::handlers::web::create_web_server(ctx, server_info, config)
    }

    fn update_server_status(&self, ctx: &ServerContext, server_id: &str, status: ServerStatus) {
        if let Ok(mut servers) = ctx.servers.write() {
            if let Some(server) = servers.get_mut(server_id) {
                server.status = status;
            }
        }
    }
}
