// Fixed src/commands/start/command.rs
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::port::is_port_available;
use crate::server::utils::validation::find_server;
use opener;

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

        let _server_id = args[0];
        let _browser_override = StartCommand::parse_browser_override(&args[1..]);

        let config = get_config()?;
        let ctx = crate::server::shared::get_shared_context();

        // FIX: config statt config_result verwenden
        self.start_server_sync(&config, ctx, args[0])
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
        // Robuste Port-Validierung
        match crate::server::utils::port::check_port_status(server_info.port) {
            crate::server::utils::port::PortStatus::Available => {
                // Port ist frei - weiter
            }
            crate::server::utils::port::PortStatus::OccupiedByUs => {
                // Port wird bereits von unserem Server verwendet
                return Ok(format!(
                    "Port {} wird bereits von Server '{}' verwendet!",
                    server_info.port, server_info.name
                ));
            }
            crate::server::utils::port::PortStatus::OccupiedByOther => {
                // Port wird von anderem Prozess verwendet
                return Ok(format!(
                    "Port {} ist von anderem Prozess belegt! Verwende anderen Port.",
                    server_info.port
                ));
            }
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

                let server_url = format!("http://127.0.0.1:{}", server_info.port);
                if config.server.auto_open_browser {
                    let url_clone = server_url.clone();
                    let server_name_clone = server_info.name.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
                        match opener::open(&url_clone) {
                            Ok(_) => {
                                log::info!(
                                    "Browser opened for '{}': {}",
                                    server_name_clone,
                                    url_clone
                                );
                            }
                            Err(e) => {
                                log::warn!(
                                    "Failed to open browser for '{}': {} (URL: {})",
                                    server_name_clone,
                                    e,
                                    url_clone
                                );
                            }
                        }
                    });
                }

                Ok(format!(
                    "Server '{}' successfully started on {} [PERSISTENT] ({}/{} running){}",
                    server_info.name,
                    server_url, // URL statt nur Port
                    running_servers + 1,
                    config.server.max_concurrent,
                    if config.server.auto_open_browser {
                        " - Browser opening..."
                    } else {
                        ""
                    }
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

    // Helper für Browser-Override Parsing
    fn parse_browser_override(args: &[&str]) -> Option<bool> {
        for arg in args {
            match *arg {
                "--no-browser" => return Some(false),
                "--browser" => return Some(true),
                _ => continue,
            }
        }
        None
    }
}
