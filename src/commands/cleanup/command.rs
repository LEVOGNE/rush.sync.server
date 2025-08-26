// CleanupCommand - src/commands/cleanup/command.rs
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Default)]
pub struct CleanupCommand;

impl CleanupCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for CleanupCommand {
    fn name(&self) -> &'static str {
        "cleanup"
    }
    fn description(&self) -> &'static str {
        "Clean up stopped or failed servers"
    }
    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("cleanup")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let ctx = crate::server::shared::get_shared_context();

        match args.first() {
            Some(&"failed") => Ok(self.cleanup_failed_servers(ctx)),
            Some(&"stopped") | None => Ok(self.cleanup_stopped_servers(ctx)),
            Some(&"all") => {
                let stopped = self.cleanup_stopped_servers(ctx);
                let failed = self.cleanup_failed_servers(ctx);
                Ok(format!("{}\n{}", stopped, failed))
            }
            _ => Err(AppError::Validation(
                "Usage: cleanup [stopped|failed|all]".to_string(),
            )),
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
        50
    }
}

impl CleanupCommand {
    fn cleanup_stopped_servers(&self, ctx: &ServerContext) -> String {
        let mut servers = ctx.servers.write().unwrap();
        let initial_count = servers.len();
        servers.retain(|_, server| server.status != ServerStatus::Stopped);
        let removed_count = initial_count - servers.len();

        if removed_count > 0 {
            format!("{} gestoppte Server entfernt", removed_count)
        } else {
            "Keine gestoppten Server zum Entfernen gefunden".to_string()
        }
    }

    fn cleanup_failed_servers(&self, ctx: &ServerContext) -> String {
        let mut servers = ctx.servers.write().unwrap();
        let initial_count = servers.len();
        servers.retain(|_, server| server.status != ServerStatus::Failed);
        let removed_count = initial_count - servers.len();

        if removed_count > 0 {
            format!("{} fehlgeschlagene Server entfernt", removed_count)
        } else {
            "Keine fehlgeschlagenen Server zum Entfernen gefunden".to_string()
        }
    }

    pub async fn shutdown_all_servers(&self, ctx: &ServerContext) -> Result<()> {
        let handles: Vec<_> = {
            let mut handles_guard = ctx.handles.write().unwrap();
            handles_guard.drain().collect()
        };

        let shutdown_futures: Vec<_> = handles
            .into_iter()
            .map(|(id, handle)| async move {
                match timeout(Duration::from_secs(5), handle.stop(true)).await {
                    Ok(_) => log::info!("Server {} stopped", id),
                    Err(_) => {
                        log::warn!("Force stopping server {}", id);
                        handle.stop(false).await;
                    }
                }
            })
            .collect();

        futures::future::join_all(shutdown_futures).await;

        {
            let mut servers = ctx.servers.write().unwrap();
            for server in servers.values_mut() {
                server.status = ServerStatus::Stopped;
            }
        }
        Ok(())
    }
}
