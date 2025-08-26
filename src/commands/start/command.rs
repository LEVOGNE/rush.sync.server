// StartCommand - src/commands/start/command.rs
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::handlers::web::create_web_server;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::{port::is_port_available, validation::find_server};
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
        "Start a web server"
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

        let ctx = crate::server::shared::get_shared_context();
        self.start_server_async(ctx, args[0])
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
        66
    }
}

impl StartCommand {
    fn start_server_async(&self, ctx: &ServerContext, identifier: &str) -> Result<String> {
        let server_info = {
            let servers = ctx.servers.read().unwrap();
            find_server(&servers, identifier)?.clone()
        };

        if server_info.status == ServerStatus::Running {
            return Ok(format!("Server '{}' läuft bereits!", server_info.name));
        }

        if !is_port_available(server_info.port) {
            return Ok(format!("Port {} bereits belegt!", server_info.port));
        }

        match self.spawn_server(ctx, server_info.clone()) {
            Ok(handle) => {
                {
                    let mut handles = ctx.handles.write().unwrap();
                    handles.insert(server_info.id.clone(), handle);
                }
                self.update_server_status(ctx, &server_info.id, ServerStatus::Running);
                Ok(format!(
                    "Server '{}' erfolgreich gestartet auf http://127.0.0.1:{}",
                    server_info.name, server_info.port
                ))
            }
            Err(e) => {
                self.update_server_status(ctx, &server_info.id, ServerStatus::Failed);
                Err(AppError::Validation(format!(
                    "Server-Start fehlgeschlagen: {}",
                    e
                )))
            }
        }
    }

    fn spawn_server(
        &self,
        ctx: &ServerContext,
        server_info: crate::server::types::ServerInfo,
    ) -> std::result::Result<actix_web::dev::ServerHandle, String> {
        create_web_server(ctx, server_info)
    }

    fn update_server_status(&self, ctx: &ServerContext, server_id: &str, status: ServerStatus) {
        if let Ok(mut servers) = ctx.servers.write() {
            if let Some(server) = servers.get_mut(server_id) {
                server.status = status;
            }
        }
    }
}
