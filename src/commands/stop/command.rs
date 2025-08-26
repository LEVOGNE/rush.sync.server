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
        "Stop a web server"
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

        let ctx = crate::server::shared::get_shared_context();
        self.stop_server(ctx, args[0])
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
        67
    }
}

impl StopCommand {
    fn stop_server(&self, ctx: &ServerContext, identifier: &str) -> Result<String> {
        let server_info = {
            let servers = ctx.servers.read().unwrap();
            find_server(&servers, identifier)?.clone()
        };

        if server_info.status != ServerStatus::Running {
            return Ok(format!(
                "Server '{}' ist nicht aktiv (Status: {})",
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
            self.shutdown_server_gracefully(handle, server_info.id.clone());
            self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);
            std::thread::sleep(Duration::from_millis(1000));
            Ok(format!("Server '{}' gestoppt", server_info.name))
        } else {
            self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);
            Ok(format!(
                "Server '{}' war bereits gestoppt",
                server_info.name
            ))
        }
    }

    fn shutdown_server_gracefully(&self, handle: actix_web::dev::ServerHandle, server_id: String) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                match timeout(Duration::from_secs(3), handle.stop(true)).await {
                    Ok(_) => log::info!("Server {} stopped gracefully", server_id),
                    Err(_) => {
                        log::warn!("Server {} shutdown timeout, forcing stop", server_id);
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
