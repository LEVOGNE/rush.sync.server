// Fixed src/commands/create/command.rs
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerInfo, ServerStatus};
use crate::server::utils::validation::validate_server_name;
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
        let config = get_config()?;

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
                Ok(port) => self.create_server(&config, ctx, Some(args[0].to_string()), Some(port)),
                Err(_) => Err(AppError::Validation("Invalid port".to_string())),
            },
            _ => Err(AppError::Validation("Too many parameters".to_string())),
        }
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
            format!("rss-{:03}", server_number)
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
            if !crate::server::utils::port::is_port_available(custom_port) {
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

        // Verzeichnis sofort bei CREATE erstellen
        if let Err(e) = crate::server::handlers::web::create_server_directory_and_files(&name, port)
        {
            return Err(AppError::Validation(format!(
                "Failed to create server directory: {}",
                e
            )));
        }

        // Add to runtime context
        ctx.servers
            .write()
            .unwrap()
            .insert(id.clone(), server_info.clone());

        // Persist to file (async)
        let registry = crate::server::shared::get_persistent_registry();
        let server_info_clone = server_info.clone();
        tokio::spawn(async move {
            if let Ok(_persistent_servers) = registry.load_servers().await {
                if let Err(e) = registry.add_server(server_info_clone).await {
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
        let used_ports: std::collections::HashSet<u16> = {
            let servers = ctx
                .servers
                .read()
                .map_err(|_| AppError::Validation("Server-Context lock poisoned".to_string()))?;
            servers.values().map(|s| s.port).collect()
        };

        let start_port = config.server.port_range_start;
        let end_port = config.server.port_range_end;

        // Sicherheitscheck für Port-Range
        if start_port >= end_port {
            return Err(AppError::Validation(format!(
                "Invalid port range: {} >= {}. Check config.",
                start_port, end_port
            )));
        }

        // Effizienter: Nicht mehr als 1000 Ports testen
        let max_attempts = ((end_port - start_port + 1) as usize).min(1000);

        for i in 0..max_attempts {
            let candidate_port = start_port + (i as u16);

            if candidate_port > end_port {
                break;
            }

            if !used_ports.contains(&candidate_port)
                && crate::server::utils::port::is_port_available(candidate_port)
            {
                return Ok(candidate_port);
            }
        }

        Err(AppError::Validation(format!(
            "No available ports in range {}-{} after {} attempts",
            start_port, end_port, max_attempts
        )))
    }

    fn find_next_server_number(&self, ctx: &ServerContext) -> u32 {
        let servers = ctx.servers.read().unwrap();
        let mut existing_numbers = Vec::new();

        for server in servers.values() {
            if let Some(number_str) = server.name.strip_prefix("rss-") {
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
