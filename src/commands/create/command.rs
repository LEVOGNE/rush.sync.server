// Fixed src/commands/create/command.rs
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerInfo, ServerStatus};
use crate::server::utils::validation::validate_server_name;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct CreateCommand;

impl CreateCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for CreateCommand {
    fn name(&self) -> &'static str {
        "create"
    }
    fn description(&self) -> &'static str {
        "Create a new web server (persistent)"
    }
    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("create")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        // DON'T use block_on - instead use spawn_blocking for config loading
        let config_result = std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(Config::load())
        })
        .join()
        .map_err(|_| AppError::Validation("Failed to load config".to_string()))??;

        let ctx = crate::server::shared::get_shared_context();

        match args.len() {
            0 => self.create_server(&config_result, ctx, None, None),
            1 => {
                if let Ok(port) = args[0].parse::<u16>() {
                    self.create_server(&config_result, ctx, None, Some(port))
                } else {
                    self.create_server(&config_result, ctx, Some(args[0].to_string()), None)
                }
            }
            2 => match args[1].parse::<u16>() {
                Ok(port) => {
                    self.create_server(&config_result, ctx, Some(args[0].to_string()), Some(port))
                }
                Err(_) => Err(AppError::Validation("Invalid port".to_string())),
            },
            _ => Err(AppError::Validation("Too many parameters".to_string())),
        }
    }

    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let config = Config::load().await?;
            let ctx = crate::server::shared::get_shared_context();

            match args.len() {
                0 => self.create_server(&config, ctx, None, None),
                1 => {
                    if let Ok(port) = args[0].parse::<u16>() {
                        self.create_server(&config, ctx, None, Some(port))
                    } else {
                        self.create_server(&config, ctx, Some(args[0].to_string()), None)
                    }
                }
                2 => match args[1].parse::<u16>() {
                    Ok(port) => {
                        self.create_server(&config, ctx, Some(args[0].to_string()), Some(port))
                    }
                    Err(_) => Err(AppError::Validation("Invalid port".to_string())),
                },
                _ => Err(AppError::Validation("Too many parameters".to_string())),
            }
        })
    }

    fn supports_async(&self) -> bool {
        true
    }
    fn priority(&self) -> u8 {
        65
    }
}

impl CreateCommand {
    fn create_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        custom_name: Option<String>,
        custom_port: Option<u16>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let has_custom_name = custom_name.is_some();
        let has_custom_port = custom_port.is_some();

        let name = if let Some(custom_name) = custom_name {
            validate_server_name(&custom_name)?;
            let servers = ctx.servers.read().unwrap();
            if servers.values().any(|s| s.name == custom_name) {
                return Err(AppError::Validation(format!(
                    "Server-Name '{}' bereits vergeben!",
                    custom_name
                )));
            }
            custom_name
        } else {
            let server_number = self.find_next_server_number(ctx);
            format!("rush-sync-server-{:03}", server_number)
        };

        let port = if let Some(custom_port) = custom_port {
            // Use configurable minimum port from config
            let min_port = config.server.port_range_start.max(1024);
            if custom_port < min_port {
                return Err(AppError::Validation(format!(
                    "Port must be >= {} (configured minimum: {})",
                    min_port, config.server.port_range_start
                )));
            }

            // Check if port is within configured range
            if custom_port > config.server.port_range_end {
                return Err(AppError::Validation(format!(
                    "Port {} exceeds configured maximum: {}",
                    custom_port, config.server.port_range_end
                )));
            }

            let servers = ctx.servers.read().unwrap();
            if servers.values().any(|s| s.port == custom_port) {
                return Err(AppError::Validation(format!(
                    "Port {} bereits verwendet!",
                    custom_port
                )));
            }
            if !self.is_port_available(custom_port) {
                return Err(AppError::Validation(format!(
                    "Port {} bereits belegt!",
                    custom_port
                )));
            }
            custom_port
        } else {
            self.find_next_available_port(config)?
        };

        // Check server count limit
        let current_server_count = ctx.servers.read().unwrap().len();
        if current_server_count >= config.server.max_concurrent {
            return Err(AppError::Validation(format!(
                "Maximum server limit reached: {}/{}. Use 'cleanup' to remove stopped servers or increase max_concurrent in config.",
                current_server_count, config.server.max_concurrent
            )));
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let server_info = ServerInfo {
            id: id.clone(),
            name: name.clone(),
            port,
            status: ServerStatus::Stopped,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            created_timestamp: timestamp,
        };

        // Add to runtime context
        ctx.servers
            .write()
            .unwrap()
            .insert(id.clone(), server_info.clone());

        // Persist to file (async)
        let registry = crate::server::shared::get_persistent_registry();
        let server_info_clone = server_info.clone();
        tokio::spawn(async move {
            if let Ok(persistent_servers) = registry.load_servers().await {
                if let Err(e) = registry
                    .add_server(persistent_servers, server_info_clone)
                    .await
                {
                    log::error!("Failed to persist server: {}", e);
                }
            }
        });

        let success_msg = if has_custom_name || has_custom_port {
            format!(
                "Custom Server created: '{}' (ID: {}) on port {} [PERSISTENT] ({}/{} servers)",
                name,
                &id[0..8],
                port,
                current_server_count + 1,
                config.server.max_concurrent
            )
        } else {
            format!(
                "Server created: '{}' (ID: {}) on port {} [PERSISTENT] ({}/{} servers)",
                name,
                &id[0..8],
                port,
                current_server_count + 1,
                config.server.max_concurrent
            )
        };

        Ok(success_msg)
    }

    // Updated to use config instead of context
    fn find_next_available_port(&self, config: &Config) -> Result<u16> {
        let ctx = crate::server::shared::get_shared_context();
        let servers = ctx.servers.read().unwrap();
        let mut used_ports: Vec<u16> = servers.values().map(|s| s.port).collect();
        used_ports.sort();

        let mut candidate_port = config.server.port_range_start;
        let max_port = config.server.port_range_end;

        loop {
            if candidate_port > max_port {
                return Err(AppError::Validation(format!(
                    "No available ports in configured range {}-{}",
                    config.server.port_range_start, config.server.port_range_end
                )));
            }

            if !used_ports.contains(&candidate_port) && self.is_port_available(candidate_port) {
                return Ok(candidate_port);
            }

            candidate_port += 1;
        }
    }

    fn is_port_available(&self, port: u16) -> bool {
        use std::net::TcpListener;
        use std::time::Duration;

        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => {
                drop(listener);
                std::thread::sleep(Duration::from_millis(10));
                TcpListener::bind(("127.0.0.1", port)).is_ok()
            }
            Err(_) => false,
        }
    }

    fn find_next_server_number(&self, ctx: &ServerContext) -> u32 {
        let servers = ctx.servers.read().unwrap();
        let mut existing_numbers = Vec::new();

        for server in servers.values() {
            if let Some(number_str) = server.name.strip_prefix("rush-sync-server-") {
                if let Ok(number) = number_str.parse::<u32>() {
                    existing_numbers.push(number);
                }
            }
        }

        existing_numbers.sort();
        let mut next_number = 1;
        for &existing in &existing_numbers {
            if existing == next_number {
                next_number += 1;
            } else {
                break;
            }
        }
        next_number
    }
}
