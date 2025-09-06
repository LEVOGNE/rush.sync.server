// src/commands/cleanup/command.rs - FIXED für konsistenten i18n-Stil
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};

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
        "Clean up servers, logs, and www files - supports confirmation and force flags"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("cleanup")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let ctx = crate::server::shared::get_shared_context();

        match args.first() {
            // FIXED: Konsistente Verwendung von get_command_translation ohne .text Suffix
            Some(&"stopped") => {
                let msg = crate::i18n::get_command_translation(
                    "system.commands.cleanup.confirm_stopped",
                    &[],
                );
                Ok(format!(
                    "__CONFIRM:__CLEANUP__cleanup --force-stopped__{}",
                    msg
                ))
            }
            Some(&"failed") => {
                let msg = crate::i18n::get_command_translation(
                    "system.commands.cleanup.confirm_failed",
                    &[],
                );
                Ok(format!(
                    "__CONFIRM:__CLEANUP__cleanup --force-failed__{}",
                    msg
                ))
            }
            Some(&"logs") => {
                let msg = crate::i18n::get_command_translation(
                    "system.commands.cleanup.confirm_logs",
                    &[],
                );
                Ok(format!(
                    "__CONFIRM:__CLEANUP__cleanup --force-logs__{}",
                    msg
                ))
            }
            Some(&"all") => {
                let msg = crate::i18n::get_command_translation(
                    "system.commands.cleanup.confirm_all",
                    &[],
                );
                Ok(format!("__CONFIRM:__CLEANUP__cleanup --force-all__{}", msg))
            }
            Some(&"www") => {
                if let Some(&server_name) = args.get(1) {
                    let msg = crate::i18n::get_command_translation(
                        "system.commands.cleanup.confirm_www_server",
                        &[server_name],
                    );
                    Ok(format!(
                        "__CONFIRM:__CLEANUP__cleanup --force-www {}__{}",
                        server_name, msg
                    ))
                } else {
                    let msg = crate::i18n::get_command_translation(
                        "system.commands.cleanup.confirm_www_all",
                        &[],
                    );
                    Ok(format!("__CONFIRM:__CLEANUP__cleanup --force-www__{}", msg))
                }
            }
            None => {
                // Default: stopped cleanup with confirmation
                let msg = crate::i18n::get_command_translation(
                    "system.commands.cleanup.confirm_stopped",
                    &[],
                );
                Ok(format!(
                    "__CONFIRM:__CLEANUP__cleanup --force-stopped__{}",
                    msg
                ))
            }

            // Force-Commands (direct execution without confirmation)
            Some(&"--force-stopped") => Ok(self.cleanup_stopped_servers(ctx)),
            Some(&"--force-failed") => Ok(self.cleanup_failed_servers(ctx)),
            Some(&"--force-logs") => {
                tokio::spawn(async move {
                    match Self::cleanup_all_server_logs().await {
                        Ok(msg) => log::info!("Log cleanup result: {}", msg),
                        Err(e) => log::error!("Log cleanup failed: {}", e),
                    }
                });
                Ok(crate::i18n::get_command_translation(
                    "system.commands.cleanup.logs_started",
                    &[],
                ))
            }
            Some(&"--force-www") => {
                if let Some(&server_name) = args.get(1) {
                    let name = server_name.to_string();
                    tokio::spawn(async move {
                        match Self::cleanup_www_by_name(&name).await {
                            Ok(msg) => log::info!("WWW cleanup result: {}", msg),
                            Err(e) => log::error!("WWW cleanup failed: {}", e),
                        }
                    });
                    Ok(crate::i18n::get_command_translation(
                        "system.commands.cleanup.www_server_started",
                        &[server_name],
                    ))
                } else {
                    tokio::spawn(async move {
                        match Self::cleanup_www_directory().await {
                            Ok(msg) => log::info!("WWW cleanup result: {}", msg),
                            Err(e) => log::error!("WWW cleanup failed: {}", e),
                        }
                    });
                    Ok(crate::i18n::get_command_translation(
                        "system.commands.cleanup.www_all_started",
                        &[],
                    ))
                }
            }
            Some(&"--force-all") => {
                // Complete cleanup now includes WWW cleanup
                let stopped = self.cleanup_stopped_servers(ctx);
                let failed = self.cleanup_failed_servers(ctx);

                // Start async cleanup tasks for www and logs
                tokio::spawn(async move {
                    // WWW cleanup is now included in "all"
                    let www_cleanup = async {
                        match Self::cleanup_www_directory().await {
                            Ok(msg) => log::info!("WWW cleanup result: {}", msg),
                            Err(e) => log::error!("WWW cleanup failed: {}", e),
                        }
                    };

                    let log_cleanup = async {
                        match Self::cleanup_all_server_logs().await {
                            Ok(msg) => log::info!("Log cleanup result: {}", msg),
                            Err(e) => log::error!("Log cleanup failed: {}", e),
                        }
                    };

                    // Both tasks run concurrently
                    tokio::join!(www_cleanup, log_cleanup);
                });

                let async_cleanup_msg = crate::i18n::get_command_translation(
                    "system.commands.cleanup.async_started",
                    &[],
                );
                Ok(format!("{}\n{}\n{}", stopped, failed, async_cleanup_msg))
            }

            _ => Err(AppError::Validation(crate::i18n::get_command_translation(
                "system.commands.cleanup.usage",
                &[],
            ))),
        }
    }

    fn priority(&self) -> u8 {
        50
    }
}

impl CleanupCommand {
    // FIXED: Alle cleanup-Methoden nutzen jetzt konsistent get_command_translation
    fn cleanup_stopped_servers(&self, ctx: &ServerContext) -> String {
        let registry = crate::server::shared::get_persistent_registry();

        tokio::spawn(async move {
            if let Ok(_servers) = registry.load_servers().await {
                if let Ok((_updated_servers, removed_count)) = registry
                    .cleanup_servers(crate::server::persistence::CleanupType::Stopped)
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

        let mut servers = ctx.servers.write().unwrap();
        let initial_count = servers.len();
        servers.retain(|_, server| server.status != ServerStatus::Stopped);
        let removed_count = initial_count - servers.len();

        if removed_count > 0 {
            crate::i18n::get_command_translation(
                "system.commands.cleanup.stopped_success",
                &[&removed_count.to_string()],
            )
        } else {
            crate::i18n::get_command_translation("system.commands.cleanup.no_stopped", &[])
        }
    }

    fn cleanup_failed_servers(&self, ctx: &ServerContext) -> String {
        let registry = crate::server::shared::get_persistent_registry();

        tokio::spawn(async move {
            if let Ok(_servers) = registry.load_servers().await {
                if let Ok((_updated_servers, removed_count)) = registry
                    .cleanup_servers(crate::server::persistence::CleanupType::Failed)
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

        let mut servers = ctx.servers.write().unwrap();
        let initial_count = servers.len();
        servers.retain(|_, server| server.status != ServerStatus::Failed);
        let removed_count = initial_count - servers.len();

        if removed_count > 0 {
            crate::i18n::get_command_translation(
                "system.commands.cleanup.failed_success",
                &[&removed_count.to_string()],
            )
        } else {
            crate::i18n::get_command_translation("system.commands.cleanup.no_failed", &[])
        }
    }

    // Alle async cleanup methods mit konsistenten i18n keys
    pub async fn cleanup_all_server_logs() -> Result<String> {
        let exe_path = std::env::current_exe().map_err(AppError::Io)?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Cannot determine executable directory".to_string())
        })?;

        let servers_dir = base_dir.join(".rss").join("servers");

        if !servers_dir.exists() {
            return Ok(crate::i18n::get_command_translation(
                "system.commands.cleanup.no_logs_dir",
                &[],
            ));
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

        Ok(crate::i18n::get_command_translation(
            "system.commands.cleanup.logs_success",
            &[&deleted_files.to_string(), &size_mb.to_string()],
        ))
    }

    pub async fn cleanup_www_directory() -> Result<String> {
        let exe_path = std::env::current_exe().map_err(AppError::Io)?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Cannot determine executable directory".to_string())
        })?;

        let www_dir = base_dir.join("www");

        if !www_dir.exists() {
            return Ok(crate::i18n::get_command_translation(
                "system.commands.cleanup.no_www_dir",
                &[],
            ));
        }

        let mut deleted_dirs = 0;
        let mut deleted_files = 0;
        let mut total_size = 0u64;

        let mut entries = tokio::fs::read_dir(&www_dir).await.map_err(AppError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(AppError::Io)? {
            let path = entry.path();
            let metadata = tokio::fs::metadata(&path).await.map_err(AppError::Io)?;

            // Skip system files (starting with .)
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }

            if metadata.is_dir() {
                total_size += Self::calculate_directory_size(&path).await.unwrap_or(0);
                tokio::fs::remove_dir_all(&path)
                    .await
                    .map_err(AppError::Io)?;
                deleted_dirs += 1;
                log::info!("Deleted directory: {}", path.display());
            } else if metadata.is_file() {
                total_size += metadata.len();
                tokio::fs::remove_file(&path).await.map_err(AppError::Io)?;
                deleted_files += 1;
                log::info!("Deleted file: {}", path.display());
            }
        }

        let size_mb = total_size / (1024 * 1024);

        Ok(crate::i18n::get_command_translation(
            "system.commands.cleanup.www_all_success",
            &[
                &deleted_dirs.to_string(),
                &deleted_files.to_string(),
                &size_mb.to_string(),
            ],
        ))
    }

    pub async fn cleanup_www_by_name(server_name: &str) -> Result<String> {
        let exe_path = std::env::current_exe().map_err(AppError::Io)?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Cannot determine executable directory".to_string())
        })?;

        let www_dir = base_dir.join("www");

        if !www_dir.exists() {
            return Ok(crate::i18n::get_command_translation(
                "system.commands.cleanup.no_www_for_server",
                &[server_name],
            ));
        }

        let mut deleted_dirs = 0;
        let mut total_size = 0u64;

        let mut entries = tokio::fs::read_dir(&www_dir).await.map_err(AppError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(AppError::Io)? {
            let path = entry.path();
            let metadata = tokio::fs::metadata(&path).await.map_err(AppError::Io)?;

            if metadata.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if Self::matches_server_name(dir_name, server_name) {
                        total_size += Self::calculate_directory_size(&path).await.unwrap_or(0);
                        tokio::fs::remove_dir_all(&path)
                            .await
                            .map_err(AppError::Io)?;
                        deleted_dirs += 1;
                        log::info!("Deleted server directory: {}", path.display());
                    }
                }
            }
        }

        let size_mb = total_size / (1024 * 1024);

        if deleted_dirs > 0 {
            Ok(crate::i18n::get_command_translation(
                "system.commands.cleanup.www_server_success",
                &[server_name, &deleted_dirs.to_string(), &size_mb.to_string()],
            ))
        } else {
            Ok(crate::i18n::get_command_translation(
                "system.commands.cleanup.no_www_for_server",
                &[server_name],
            ))
        }
    }

    async fn calculate_directory_size(dir: &std::path::Path) -> Result<u64> {
        let mut total_size = 0u64;
        let mut stack = vec![dir.to_path_buf()];

        while let Some(current_dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&current_dir)
                .await
                .map_err(AppError::Io)?;

            while let Some(entry) = entries.next_entry().await.map_err(AppError::Io)? {
                let metadata = entry.metadata().await.map_err(AppError::Io)?;

                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    stack.push(entry.path());
                }
            }
        }

        Ok(total_size)
    }

    fn matches_server_name(dir_name: &str, server_name: &str) -> bool {
        if dir_name == server_name {
            return true;
        }

        if dir_name.starts_with(&format!("{}-[", server_name)) {
            return true;
        }

        if dir_name.contains(server_name) {
            if dir_name.contains('[') && dir_name.ends_with(']') {
                if let Some(bracket_start) = dir_name.rfind('[') {
                    if let Some(port_str) = dir_name.get(bracket_start + 1..dir_name.len() - 1) {
                        return port_str.parse::<u16>().is_ok();
                    }
                }
            }
        }

        false
    }
}
