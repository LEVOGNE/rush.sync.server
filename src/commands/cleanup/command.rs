use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
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
        "Clean up stopped or failed servers (persistent)"
    }
    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("cleanup")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let ctx = crate::server::shared::get_shared_context();

        match args.first() {
            Some(&"failed") => Ok(self.cleanup_failed_servers(ctx)),
            Some(&"stopped") | None => Ok(self.cleanup_stopped_servers(ctx)),
            Some(&"logs") => {
                tokio::spawn(async move {
                    match Self::cleanup_all_server_logs().await {
                        Ok(msg) => log::info!("Log cleanup result: {}", msg),
                        Err(e) => log::error!("Log cleanup failed: {}", e),
                    }
                });
                Ok("Server-Log-Bereinigung gestartet...".to_string())
            }
            Some(&"all") => {
                let stopped = self.cleanup_stopped_servers(ctx);
                let failed = self.cleanup_failed_servers(ctx);

                tokio::spawn(async move {
                    match Self::cleanup_all_server_logs().await {
                        Ok(msg) => log::info!("Log cleanup result: {}", msg),
                        Err(e) => log::error!("Log cleanup failed: {}", e),
                    }
                });

                Ok(format!(
                    "{}\n{}\nServer-Logs werden bereinigt...",
                    stopped, failed
                ))
            }
            _ => Err(AppError::Validation(
                "Usage: cleanup [stopped|failed|logs|all]".to_string(),
            )),
        }
    }

    fn priority(&self) -> u8 {
        50
    }
}

impl CleanupCommand {
    fn cleanup_stopped_servers(&self, ctx: &ServerContext) -> String {
        let registry = crate::server::shared::get_persistent_registry();

        // Async cleanup mit Persistence
        tokio::spawn(async move {
            if let Ok(servers) = registry.load_servers().await {
                if let Ok((_updated_servers, removed_count)) = registry
                    .cleanup_servers(servers, crate::server::persistence::CleanupType::Stopped)
                    .await
                {
                    if removed_count > 0 {
                        log::info!(
                            "Removed {} stopped servers from persistent registry",
                            removed_count
                        );
                    }
                }
            }
        });

        // Sofortige Runtime-Cleanup
        let mut servers = ctx.servers.write().unwrap();
        let initial_count = servers.len();
        servers.retain(|_, server| server.status != ServerStatus::Stopped);
        let removed_count = initial_count - servers.len();

        if removed_count > 0 {
            format!(
                "{} gestoppte Server entfernt (persistent gespeichert)",
                removed_count
            )
        } else {
            "Keine gestoppten Server zum Entfernen gefunden".to_string()
        }
    }

    fn cleanup_failed_servers(&self, ctx: &ServerContext) -> String {
        let registry = crate::server::shared::get_persistent_registry();

        // Async cleanup mit Persistence
        tokio::spawn(async move {
            if let Ok(servers) = registry.load_servers().await {
                if let Ok((_updated_servers, removed_count)) = registry
                    .cleanup_servers(servers, crate::server::persistence::CleanupType::Failed)
                    .await
                {
                    if removed_count > 0 {
                        log::info!(
                            "Removed {} failed servers from persistent registry",
                            removed_count
                        );
                    }
                }
            }
        });

        // Sofortige Runtime-Cleanup
        let mut servers = ctx.servers.write().unwrap();
        let initial_count = servers.len();
        servers.retain(|_, server| server.status != ServerStatus::Failed);
        let removed_count = initial_count - servers.len();

        if removed_count > 0 {
            format!(
                "{} fehlgeschlagene Server entfernt (persistent gespeichert)",
                removed_count
            )
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

        // Persistenz-Update für alle Server
        let registry = crate::server::shared::get_persistent_registry();
        if let Ok(mut persistent_servers) = registry.load_servers().await {
            for server in persistent_servers.values_mut() {
                server.status = ServerStatus::Stopped;
            }
            let _ = registry.save_servers(&persistent_servers).await;
        }

        Ok(())
    }

    pub async fn cleanup_all_server_logs() -> Result<String> {
        let exe_path = std::env::current_exe().map_err(AppError::Io)?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Cannot determine executable directory".to_string())
        })?;

        let servers_dir = base_dir.join(".rss").join("servers");

        if !servers_dir.exists() {
            return Ok("Kein servers/ Verzeichnis gefunden".to_string());
        }

        let mut deleted_files = 0;
        let mut total_size = 0u64;

        let mut entries = tokio::fs::read_dir(&servers_dir)
            .await
            .map_err(AppError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(AppError::Io)? {
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "log" || extension == "gz" {
                        if let Ok(metadata) = tokio::fs::metadata(&path).await {
                            total_size += metadata.len();
                        }

                        tokio::fs::remove_file(&path).await.map_err(AppError::Io)?;
                        deleted_files += 1;

                        log::info!("Deleted log file: {}", path.display());
                    }
                }
            }
        }

        let size_mb = total_size / (1024 * 1024);

        Ok(format!(
            "Server-Logs bereinigt: {} Dateien gelöscht, {}MB freigegeben",
            deleted_files, size_mb
        ))
    }
}
