// NEW: src/commands/recovery/command.rs
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
        80 // Hohe Priorität für Recovery
    }
}

impl RecoveryCommand {
    // ✅ AUTO-RECOVERY: Analysiert und repariert alle inkonsistenten Server
    fn auto_recover(&self, ctx: &ServerContext) -> String {
        let mut fixes = Vec::new();
        let registry = crate::server::shared::get_persistent_registry();

        // 1. Handle-Status-Synchronisation
        let (orphaned_handles, missing_handles) = {
            let servers = ctx.servers.read().unwrap();
            let handles = ctx.handles.read().unwrap();

            // Handles ohne entsprechende Server (Orphaned)
            let orphaned: Vec<String> = handles
                .keys()
                .filter(|id| !servers.contains_key(*id))
                .cloned()
                .collect();

            // Running-Server ohne Handle (Missing)
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

        // 2. Port-Status-Validierung
        let port_fixes = self.validate_and_fix_ports(ctx);
        fixes.extend(port_fixes);

        // 3. Orphaned Handles bereinigen
        if !orphaned_handles.is_empty() {
            let count = orphaned_handles.len();
            for handle_id in orphaned_handles {
                let mut handles = ctx.handles.write().unwrap();
                if let Some(handle) = handles.remove(&handle_id) {
                    // Handle graceful stoppen
                    tokio::spawn(async move {
                        let _ = handle.stop(true).await;
                    });
                }
            }
            fixes.push(format!("🧹 {} orphaned handles cleaned", count));
        }

        // 4. Missing Handles reparieren
        if !missing_handles.is_empty() {
            for server_id in &missing_handles {
                self.fix_missing_handle(ctx, &server_id);
            }
            fixes.push(format!(
                "🔧 {} missing handles fixed",
                missing_handles.len()
            ));
        }

        // 5. Persistence synchronisieren
        tokio::spawn(async move {
            if let Ok(persistent_servers) = registry.load_servers().await {
                // Hier würde man die aktuelle Runtime-State in die Persistence schreiben
                let _ = registry.save_servers(&persistent_servers).await;
            }
        });

        if fixes.is_empty() {
            "✅ All servers are in consistent state".to_string()
        } else {
            format!("🛠️ Recovery completed:\n{}", fixes.join("\n"))
        }
    }

    // ✅ EINZELNEN SERVER REPARIEREN
    fn recover_single_server(&self, ctx: &ServerContext, identifier: &str) -> String {
        let servers = ctx.servers.read().unwrap();

        // Server finden
        let server_info = match servers
            .values()
            .find(|s| s.id.starts_with(identifier) || s.name == identifier)
        {
            Some(server) => server.clone(),
            None => return format!("❌ Server '{}' not found", identifier),
        };

        drop(servers); // Lock freigeben

        let fixes = self.diagnose_and_fix_server(ctx, &server_info);

        if fixes.is_empty() {
            format!(
                "✅ Server '{}' is already in consistent state",
                server_info.name
            )
        } else {
            format!(
                "🛠️ Fixed server '{}':\n{}",
                server_info.name,
                fixes.join("\n")
            )
        }
    }

    // ✅ ALLE SERVER DURCHGEHEN
    fn recover_all_servers(&self, ctx: &ServerContext) -> String {
        let mut total_fixes = Vec::new();
        let servers: Vec<_> = {
            let servers = ctx.servers.read().unwrap();
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
            "✅ All servers are in consistent state".to_string()
        } else {
            format!("🛠️ Recovery results:\n{}", total_fixes.join("\n"))
        }
    }

    // ✅ SERVER DIAGNOSE UND REPARATUR
    fn diagnose_and_fix_server(
        &self,
        ctx: &ServerContext,
        server_info: &crate::server::types::ServerInfo,
    ) -> Vec<String> {
        let mut fixes = Vec::new();

        // Handle-Status prüfen
        let has_handle = {
            let handles = ctx.handles.read().unwrap();
            handles.contains_key(&server_info.id)
        };

        // Port-Status prüfen
        let port_available = is_port_available(server_info.port);

        match (server_info.status, has_handle, port_available) {
            // INKONSISTENZ: Server soll laufen, aber kein Handle
            (ServerStatus::Running, false, _) => {
                if port_available {
                    // Port frei, aber Status Running → Korrigieren zu Stopped
                    self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);
                    fixes.push("Status: Running → Stopped (no handle, port free)".to_string());
                } else {
                    // Port belegt, Handle fehlt → Neustart versuchen oder Failed setzen
                    self.update_server_status(ctx, &server_info.id, ServerStatus::Failed);
                    fixes.push("Status: Running → Failed (no handle, port occupied)".to_string());
                }
            }

            // INKONSISTENZ: Server hat Handle, aber Status nicht Running
            (status, true, _) if status != ServerStatus::Running => {
                self.update_server_status(ctx, &server_info.id, ServerStatus::Running);
                fixes.push(format!("Status: {} → Running (handle exists)", status));
            }

            // INKONSISTENZ: Server Failed, aber Port ist frei
            (ServerStatus::Failed, false, true) => {
                self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);
                fixes.push("Status: Failed → Stopped (port now free)".to_string());
            }

            _ => {
                // Server ist konsistent
            }
        }

        fixes
    }

    // ✅ PORT-VALIDIERUNG FÜR ALLE SERVER
    fn validate_and_fix_ports(&self, ctx: &ServerContext) -> Vec<String> {
        let mut fixes = Vec::new();
        let servers: Vec<_> = {
            let servers = ctx.servers.read().unwrap();
            servers.values().cloned().collect()
        };

        for server_info in servers {
            let port_available = is_port_available(server_info.port);
            let has_handle = {
                let handles = ctx.handles.read().unwrap();
                handles.contains_key(&server_info.id)
            };

            // Logik-Matrix für Port-Fixes
            match (server_info.status, has_handle, port_available) {
                (ServerStatus::Running, true, false) => {
                    // OK: Server läuft, Handle da, Port belegt
                }
                (ServerStatus::Running, false, false) => {
                    // Problem: Server soll laufen, aber kein Handle und Port belegt
                    self.update_server_status(ctx, &server_info.id, ServerStatus::Failed);
                    fixes.push(format!(
                        "Fixed '{}': Running → Failed (orphaned)",
                        server_info.name
                    ));
                }
                (ServerStatus::Running, false, true) => {
                    // Problem: Server soll laufen, kein Handle, Port frei
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

    // ✅ MISSING HANDLE REPARIEREN
    fn fix_missing_handle(&self, ctx: &ServerContext, server_id: &str) {
        // Server auf Stopped setzen, da wir kein Handle haben
        self.update_server_status(ctx, server_id, ServerStatus::Stopped);

        // Async persistence update
        let server_id_clone = server_id.to_string();
        tokio::spawn(async move {
            crate::server::shared::persist_server_update(&server_id_clone, ServerStatus::Stopped)
                .await;
        });
    }

    // ✅ STATUS UPDATE
    fn update_server_status(&self, ctx: &ServerContext, server_id: &str, status: ServerStatus) {
        if let Ok(mut servers) = ctx.servers.write() {
            if let Some(server) = servers.get_mut(server_id) {
                server.status = status;
            }
        }
    }
}
