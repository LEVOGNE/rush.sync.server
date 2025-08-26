// CreateCommand - src/commands/create/command.rs
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerInfo, ServerStatus};
use crate::server::utils::port::{find_next_available_port, is_port_available};
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
        "Create a new web server"
    }
    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("create")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let ctx = crate::server::shared::get_shared_context();

        match args.len() {
            0 => self.create_server(ctx, None, None),
            1 => {
                if let Ok(port) = args[0].parse::<u16>() {
                    self.create_server(ctx, None, Some(port))
                } else {
                    self.create_server(ctx, Some(args[0].to_string()), None)
                }
            }
            2 => match args[1].parse::<u16>() {
                Ok(port) => self.create_server(ctx, Some(args[0].to_string()), Some(port)),
                Err(_) => Err(AppError::Validation("Invalid port".to_string())),
            },
            _ => Err(AppError::Validation("Too many parameters".to_string())),
        }
    }

    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move { self.execute_sync(args) })
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
            if custom_port < 1024 {
                return Err(AppError::Validation("Port muss >= 1024 sein!".to_string()));
            }
            let servers = ctx.servers.read().unwrap();
            if servers.values().any(|s| s.port == custom_port) {
                return Err(AppError::Validation(format!(
                    "Port {} bereits verwendet!",
                    custom_port
                )));
            }
            if !is_port_available(custom_port) {
                return Err(AppError::Validation(format!(
                    "Port {} bereits belegt!",
                    custom_port
                )));
            }
            custom_port
        } else {
            find_next_available_port(ctx)?
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

        ctx.servers.write().unwrap().insert(id.clone(), server_info);

        let success_msg = if has_custom_name || has_custom_port {
            format!(
                "Custom Server erstellt: '{}' (ID: {}) auf Port {}",
                name,
                &id[0..8],
                port
            )
        } else {
            format!(
                "Server erstellt: '{}' (ID: {}) auf Port {}",
                name,
                &id[0..8],
                port
            )
        };

        Ok(success_msg)
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
