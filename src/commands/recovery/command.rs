use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::port::is_port_available;

#[derive(Debug, Default)]
pub struct RecoveryCommand;

impl RecoveryCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for RecoveryCommand {
    fn name(&self) -> &'static str {
        "recover"
    }

    fn description(&self) -> &'static str {
        "Recover and fix server status inconsistencies"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        cmd.starts_with("recover") || cmd.starts_with("fix") || cmd == "status-fix"
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let ctx = crate::server::shared::get_shared_context();

        match args.first() {
            Some(&"all") => Ok(self.recover_all_servers(ctx)),
            Some(&server_id) => Ok(self.recover_single_server(ctx, server_id)),
            None => Ok(self.auto_recover(ctx)),
        }
    }

    fn priority(&self) -> u8 {
        80
    }
}

impl RecoveryCommand {
    /// Analyzes and repairs all inconsistent servers
    fn auto_recover(&self, ctx: &ServerContext) -> String {
        let mut fixes = Vec::new();
        let registry = crate::server::shared::get_persistent_registry();

        // 1. Handle-status synchronization
        let (orphaned_handles, missing_handles) = {
            let servers = match ctx.servers.read() {
                Ok(s) => s,
                Err(e) => {
                    log::error!("servers lock poisoned: {}", e);
                    return "Error: lock poisoned".to_string();
                }
            };
            let handles = match ctx.handles.read() {
                Ok(h) => h,
                Err(e) => {
                    log::error!("handles lock poisoned: {}", e);
                    return "Error: lock poisoned".to_string();
                }
            };

            // Handles without corresponding servers (orphaned)
            let orphaned: Vec<String> = handles
                .keys()
                .filter(|id| !servers.contains_key(*id))
                .cloned()
                .collect();

            // Running servers without a handle (missing)
            let missing: Vec<String> = servers
                .iter()
                .filter_map(|(id, server)| {
                    if server.status == ServerStatus::Running && !handles.contains_key(id) {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            (orphaned, missing)
        };

        // 2. Port status validation
        let port_fixes = self.validate_and_fix_ports(ctx);
        fixes.extend(port_fixes);

        // 3. Clean up orphaned handles
        if !orphaned_handles.is_empty() {
            let count = orphaned_handles.len();
            for handle_id in orphaned_handles {
                let mut handles = match ctx.handles.write() {
                    Ok(h) => h,
                    Err(e) => {
                        log::error!("handles lock poisoned: {}", e);
                        continue;
                    }
                };
                if let Some(handle) = handles.remove(&handle_id) {
                    // Stop handle gracefully
                    tokio::spawn(async move {
                        let _ = handle.stop(true).await;
                    });
                }
            }
            fixes.push(format!("{} orphaned handles cleaned", count));
        }

        // 4. Repair missing handles
        if !missing_handles.is_empty() {
            for server_id in &missing_handles {
                self.fix_missing_handle(ctx, server_id);
            }
            fixes.push(format!("{} missing handles fixed", missing_handles.len()));
        }

        // 5. Synchronize persistence
        tokio::spawn(async move {
            if let Ok(persistent_servers) = registry.load_servers().await {
                let _ = registry.save_servers(&persistent_servers).await;
            }
        });

        if fixes.is_empty() {
            "All servers are in consistent state".to_string()
        } else {
            format!("Recovery completed:\n{}", fixes.join("\n"))
        }
    }

    fn recover_single_server(&self, ctx: &ServerContext, identifier: &str) -> String {
        let servers = match ctx.servers.read() {
            Ok(s) => s,
            Err(e) => {
                log::error!("servers lock poisoned: {}", e);
                return "Error: lock poisoned".to_string();
            }
        };

        // Find server
        let server_info = match servers
            .values()
            .find(|s| s.id.starts_with(identifier) || s.name == identifier)
        {
            Some(server) => server.clone(),
            None => return format!("Server '{}' not found", identifier),
        };

        drop(servers); // Release lock

        let fixes = self.diagnose_and_fix_server(ctx, &server_info);

        if fixes.is_empty() {
            format!(
                "Server '{}' is already in consistent state",
                server_info.name
            )
        } else {
            format!("Fixed server '{}':\n{}", server_info.name, fixes.join("\n"))
        }
    }

    fn recover_all_servers(&self, ctx: &ServerContext) -> String {
        let mut total_fixes = Vec::new();
        let servers: Vec<_> = {
            let servers = match ctx.servers.read() {
                Ok(s) => s,
                Err(e) => {
                    log::error!("servers lock poisoned: {}", e);
                    return "Error: lock poisoned".to_string();
                }
            };
            servers.values().cloned().collect()
        };

        for server_info in servers {
            let fixes = self.diagnose_and_fix_server(ctx, &server_info);
            if !fixes.is_empty() {
                total_fixes.push(format!(
                    "Server '{}': {}",
                    server_info.name,
                    fixes.join(", ")
                ));
            }
        }

        if total_fixes.is_empty() {
            "All servers are in consistent state".to_string()
        } else {
            format!("Recovery results:\n{}", total_fixes.join("\n"))
        }
    }

    fn diagnose_and_fix_server(
        &self,
        ctx: &ServerContext,
        server_info: &crate::server::types::ServerInfo,
    ) -> Vec<String> {
        let mut fixes = Vec::new();

        let has_handle = {
            let handles = match ctx.handles.read() {
                Ok(h) => h,
                Err(_) => return fixes,
            };
            handles.contains_key(&server_info.id)
        };

        // Check port status (use 127.0.0.1 as conservative check)
        let port_available = is_port_available(server_info.port, "127.0.0.1");

        match (server_info.status, has_handle, port_available) {
            // Inconsistency: server should be running but has no handle
            (ServerStatus::Running, false, _) => {
                if port_available {
                    // Port free but status Running -> correct to Stopped
                    self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);
                    fixes.push("Status: Running → Stopped (no handle, port free)".to_string());
                } else {
                    // Port occupied but handle missing -> mark as Failed
                    self.update_server_status(ctx, &server_info.id, ServerStatus::Failed);
                    fixes.push("Status: Running → Failed (no handle, port occupied)".to_string());
                }
            }

            // Inconsistency: server has handle but status is not Running
            (status, true, _) if status != ServerStatus::Running => {
                self.update_server_status(ctx, &server_info.id, ServerStatus::Running);
                fixes.push(format!("Status: {} → Running (handle exists)", status));
            }

            // Inconsistency: server Failed but port is free
            (ServerStatus::Failed, false, true) => {
                self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);
                fixes.push("Status: Failed → Stopped (port now free)".to_string());
            }

            _ => {
                // Server is consistent
            }
        }

        fixes
    }

    fn validate_and_fix_ports(&self, ctx: &ServerContext) -> Vec<String> {
        let mut fixes = Vec::new();
        let servers: Vec<_> = {
            let servers = match ctx.servers.read() {
                Ok(s) => s,
                Err(_) => return fixes,
            };
            servers.values().cloned().collect()
        };

        for server_info in servers {
            let port_available = is_port_available(server_info.port, "127.0.0.1");
            let has_handle = {
                let handles = match ctx.handles.read() {
                    Ok(h) => h,
                    Err(_) => continue,
                };
                handles.contains_key(&server_info.id)
            };

            // Port fix decision matrix
            match (server_info.status, has_handle, port_available) {
                (ServerStatus::Running, true, false) => {
                    // OK: server running, handle present, port occupied
                }
                (ServerStatus::Running, false, false) => {
                    // Server should be running but no handle and port is occupied
                    self.update_server_status(ctx, &server_info.id, ServerStatus::Failed);
                    fixes.push(format!(
                        "Fixed '{}': Running → Failed (orphaned)",
                        server_info.name
                    ));
                }
                (ServerStatus::Running, false, true) => {
                    // Server should be running but no handle and port is free
                    self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);
                    fixes.push(format!(
                        "Fixed '{}': Running → Stopped (no handle)",
                        server_info.name
                    ));
                }
                _ => {}
            }
        }

        fixes
    }

    fn fix_missing_handle(&self, ctx: &ServerContext, server_id: &str) {
        // Set server to Stopped since we have no handle
        self.update_server_status(ctx, server_id, ServerStatus::Stopped);

        // Async persistence update
        let server_id_clone = server_id.to_string();
        tokio::spawn(async move {
            crate::server::shared::persist_server_update(&server_id_clone, ServerStatus::Stopped)
                .await;
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
