// Enhanced src/commands/create/command.rs - BULK CREATION SUPPORT
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
        "Create web server(s) - supports bulk creation"
    }
    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("create")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let config = get_config()?;
        let ctx = crate::server::shared::get_shared_context();

        // Parse arguments for different creation modes
        match self.parse_creation_args(args) {
            CreationMode::Single { name, port } => {
                self.create_single_server(&config, ctx, name, port)
            }
            CreationMode::BulkAuto { count } => {
                self.create_bulk_servers(&config, ctx, count, None, None)
            }
            CreationMode::BulkWithBase {
                base_name,
                base_port,
                count,
            } => self.create_bulk_servers(&config, ctx, count, Some(base_name), Some(base_port)),
            CreationMode::Invalid(error) => Err(AppError::Validation(error)),
        }
    }

    fn priority(&self) -> u8 {
        65
    }
}

#[derive(Debug)]
enum CreationMode {
    Single {
        name: Option<String>,
        port: Option<u16>,
    },
    BulkAuto {
        count: u32,
    },
    BulkWithBase {
        base_name: String,
        base_port: u16,
        count: u32,
    },
    Invalid(String),
}

impl CreateCommand {
    // Argument parsing logic
    fn parse_creation_args(&self, args: &[&str]) -> CreationMode {
        match args.len() {
            0 => CreationMode::Single {
                name: None,
                port: None,
            },

            1 => {
                // Erst auf Port prüfen (4-5 Stellen), dann auf Count (1-2 Stellen)
                if let Ok(port) = args[0].parse::<u16>() {
                    if port >= 1000 {
                        // "create 8080" -> Single server with port
                        CreationMode::Single {
                            name: None,
                            port: Some(port),
                        }
                    } else if port > 0 && port <= 50 {
                        // "create 5" -> Bulk creation with count
                        CreationMode::BulkAuto { count: port as u32 }
                    } else {
                        CreationMode::Invalid("Invalid port or count".to_string())
                    }
                } else {
                    // "create myserver" -> Single server with name
                    CreationMode::Single {
                        name: Some(args[0].to_string()),
                        port: None,
                    }
                }
            }

            2 => {
                // "create myserver 8080" -> Single with name and port
                if let Ok(port) = args[1].parse::<u16>() {
                    CreationMode::Single {
                        name: Some(args[0].to_string()),
                        port: Some(port),
                    }
                } else {
                    CreationMode::Invalid("Invalid port number".to_string())
                }
            }

            3 => {
                // "create myserver 8080 5" -> Bulk with base name, port, and count
                if let (Ok(port), Ok(count)) = (args[1].parse::<u16>(), args[2].parse::<u32>()) {
                    if count == 0 {
                        return CreationMode::Invalid("Count must be > 0".to_string());
                    }
                    if count > 50 {
                        return CreationMode::Invalid(
                            "Maximum 50 servers per bulk operation".to_string(),
                        );
                    }
                    CreationMode::BulkWithBase {
                        base_name: args[0].to_string(),
                        base_port: port,
                        count,
                    }
                } else {
                    CreationMode::Invalid("Invalid port or count".to_string())
                }
            }

            _ => CreationMode::Invalid(
                "Too many arguments. Usage: create [name] [port] [count]".to_string(),
            ),
        }
    }

    // Single server creation (existing logic)
    fn create_single_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        custom_name: Option<String>,
        custom_port: Option<u16>,
    ) -> Result<String> {
        let result = self.create_server_internal(config, ctx, custom_name, custom_port)?;
        Ok(format!("Server created: {}", result.summary))
    }

    // Bulk server creation
    fn create_bulk_servers(
        &self,
        config: &Config,
        ctx: &ServerContext,
        count: u32,
        base_name: Option<String>,
        base_port: Option<u16>,
    ) -> Result<String> {
        let initial_server_count = ctx.servers.read().unwrap().len();

        // Check if bulk creation would exceed limits
        if initial_server_count + (count as usize) > config.server.max_concurrent {
            return Err(AppError::Validation(format!(
                "Bulk creation would exceed server limit: {} + {} > {} (max_concurrent)",
                initial_server_count, count, config.server.max_concurrent
            )));
        }

        let mut created_servers = Vec::new();
        let mut failed_servers = Vec::new();

        for i in 0..count {
            let (name, port) =
                if let (Some(ref base_name), Some(base_port)) = (&base_name, base_port) {
                    // Use base name with suffix and increment port
                    let name = format!("{}-{:03}", base_name, i + 1);
                    let port = base_port + (i as u16);
                    (Some(name), Some(port))
                } else {
                    // Use automatic naming and port assignment (both None for auto)
                    (None, None)
                };

            match self.create_server_internal(config, ctx, name, port) {
                Ok(result) => {
                    created_servers.push(result);
                }
                Err(e) => {
                    failed_servers.push(format!("Server {}: {}", i + 1, e));

                    // Stop on critical errors, continue on port conflicts
                    if !e.to_string().contains("bereits") && !e.to_string().contains("occupied") {
                        break;
                    }
                }
            }
        }

        // Format results
        let mut result = format!(
            "Bulk creation completed: {} of {} servers created",
            created_servers.len(),
            count
        );

        if !created_servers.is_empty() {
            result.push_str("\n\nCreated servers:");
            for server in &created_servers {
                result.push_str(&format!("\n  {} - {}", server.name, server.summary));
            }
        }

        if !failed_servers.is_empty() {
            result.push_str("\n\nFailed:");
            for failure in &failed_servers {
                result.push_str(&format!("\n  {}", failure));
            }
        }

        let final_count = ctx.servers.read().unwrap().len();
        result.push_str(&format!(
            "\n\nTotal servers: {}/{}",
            final_count, config.server.max_concurrent
        ));

        Ok(result)
    }

    // Internal server creation logic (extracted from original)
    fn create_server_internal(
        &self,
        config: &Config,
        ctx: &ServerContext,
        custom_name: Option<String>,
        custom_port: Option<u16>,
    ) -> Result<ServerCreationResult> {
        let id = Uuid::new_v4().to_string();
        let has_custom_name = custom_name.is_some();

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
            let min_port = config.server.port_range_start.max(1024);
            if custom_port < min_port {
                return Err(AppError::Validation(format!(
                    "Port must be >= {} (configured minimum: {})",
                    min_port, config.server.port_range_start
                )));
            }

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

        // Create server directory and files
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

        let summary = if has_custom_name {
            format!(
                "'{}' (ID: {}) on port {} [PERSISTENT]",
                name,
                &id[0..8],
                port
            )
        } else {
            format!(
                "'{}' (ID: {}) on port {} [PERSISTENT]",
                name,
                &id[0..8],
                port
            )
        };

        Ok(ServerCreationResult { name, summary })
    }

    // Existing helper methods (unchanged)
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

        if start_port >= end_port {
            return Err(AppError::Validation(format!(
                "Invalid port range: {} >= {}. Check config.",
                start_port, end_port
            )));
        }

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

#[derive(Debug)]
struct ServerCreationResult {
    name: String,
    summary: String,
}
