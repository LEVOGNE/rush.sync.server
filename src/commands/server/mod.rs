// =====================================================
// FILE: src/commands/server/mod.rs - SERVER CLI COMMANDS
// =====================================================

use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::{ServerManager, ServerMode};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

// Global Server Manager (Singleton)
lazy_static::lazy_static! {
    static ref SERVER_MANAGER: Arc<Mutex<Option<ServerManager>>> = Arc::new(Mutex::new(None));
}

/// Holt oder erstellt Server Manager
fn get_server_manager() -> ServerManager {
    let mut manager_guard = SERVER_MANAGER.lock().unwrap_or_else(|poisoned| {
        log::warn!("Recovered from poisoned mutex");
        poisoned.into_inner()
    });

    if manager_guard.is_none() {
        *manager_guard = Some(ServerManager::new());
    }

    manager_guard.as_ref().unwrap().clone()
}

/// Server-Management Command
#[derive(Debug)]
pub struct ServerCommand;

impl Command for ServerCommand {
    fn name(&self) -> &'static str {
        "server"
    }

    fn description(&self) -> &'static str {
        "Manage Actix-Web server instances (create, start, stop, status)"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("server")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        // Für Server-Management brauchen wir async, also nutzen wir eine einfache Synchronisation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::Terminal(format!("Failed to create async runtime: {}", e)))?;

        rt.block_on(self.execute_async_internal(args))
    }

    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(self.execute_async_internal(args))
    }

    fn supports_async(&self) -> bool {
        true
    }

    fn priority(&self) -> u8 {
        90 // Hohe Priorität für Server-Management
    }
}

impl ServerCommand {
    async fn execute_async_internal(&self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Ok(self.show_help());
        }

        let manager = get_server_manager();

        match args[0].to_lowercase().as_str() {
            "create" => self.handle_create(&manager, &args[1..]).await,
            "start" => self.handle_start(&manager, &args[1..]).await,
            "stop" => self.handle_stop(&manager, &args[1..]).await,
            "delete" | "remove" => self.handle_delete(&manager, &args[1..]).await,
            "status" => self.handle_status(&manager, &args[1..]).await,
            "list" => self.handle_list(&manager).await,
            "logs" => self.handle_logs(&args[1..]).await,
            "--help" | "-h" => Ok(self.show_help()),
            _ => Ok(format!(
                "❌ Unknown server command: {}\n\n{}",
                args[0],
                self.show_help()
            )),
        }
    }

    /// Server erstellen
    async fn handle_create(&self, manager: &ServerManager, args: &[&str]) -> Result<String> {
        let mut port = 8080;
        let mut mode = ServerMode::Dev;

        // Argumente parsen
        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "--port" | "-p" => {
                    if i + 1 < args.len() {
                        port = args[i + 1]
                            .parse()
                            .map_err(|_| AppError::Validation("Invalid port number".to_string()))?;
                        i += 2;
                    } else {
                        return Err(AppError::Validation("--port requires a value".to_string()));
                    }
                }
                "--mode" | "-m" => {
                    if i + 1 < args.len() {
                        mode = match args[i + 1].to_lowercase().as_str() {
                            "dev" | "development" => ServerMode::Dev,
                            "prod" | "production" => ServerMode::Prod,
                            _ => {
                                return Err(AppError::Validation(
                                    "Mode must be 'dev' or 'prod'".to_string(),
                                ))
                            }
                        };
                        i += 2;
                    } else {
                        return Err(AppError::Validation("--mode requires a value".to_string()));
                    }
                }
                "--help" | "-h" => {
                    return Ok(self.show_create_help());
                }
                _ => {
                    return Err(AppError::Validation(format!(
                        "Unknown create option: {}",
                        args[i]
                    )));
                }
            }
        }

        // Port verfügbar?
        if port < 1024 {
            return Err(AppError::Validation(
                "Ports < 1024 require root privileges".to_string(),
            ));
        }

        // Auto-Port-Finding wenn gewünscht
        if port == 0 {
            port = manager.find_free_port(8080).await;
        }

        // Server erstellen
        let server_id = manager.create_server(port, mode).await?;

        Ok(format!(
            "✅ Server created successfully!\n\n\
            📊 Server Details:\n\
            • ID: {}\n\
            • Port: {}\n\
            • Mode: {}\n\
            • Status: Stopped\n\n\
            🚀 Next steps:\n\
            • Start server: server start {}\n\
            • Check status: server status {}\n\
            • Open in browser: http://localhost:{}",
            server_id, port, mode, server_id, server_id, port
        ))
    }

    /// Server starten
    async fn handle_start(&self, manager: &ServerManager, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(AppError::Validation(
                "Usage: server start <server-id>".to_string(),
            ));
        }

        let server_id = args[0];
        manager.start_server(server_id).await?;

        Ok(format!("🚀 Server {} started successfully!", server_id))
    }

    /// Server stoppen
    async fn handle_stop(&self, manager: &ServerManager, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(AppError::Validation(
                "Usage: server stop <server-id>".to_string(),
            ));
        }

        let server_id = args[0];
        manager.stop_server(server_id).await?;

        Ok(format!("🛑 Server {} stopped successfully!", server_id))
    }

    /// Server löschen
    async fn handle_delete(&self, manager: &ServerManager, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(AppError::Validation(
                "Usage: server delete <server-id>".to_string(),
            ));
        }

        let server_id = args[0];
        manager.delete_server(server_id).await?;

        Ok(format!("🗑️ Server {} deleted successfully!", server_id))
    }

    /// Server-Status anzeigen
    async fn handle_status(&self, manager: &ServerManager, args: &[&str]) -> Result<String> {
        if args.is_empty() || args[0] == "--all" || args[0] == "-a" {
            // Alle Server anzeigen
            let servers = manager.list_servers().await;
            if servers.is_empty() {
                return Ok(
                    "📭 No servers found.\n\n💡 Create one with: server create --port 8080"
                        .to_string(),
                );
            }

            let stats = manager.get_stats().await;
            let mut result = format!(
                "📊 Server Overview ({} total, {} running, {} stopped)\n\n",
                stats.get("total").unwrap_or(&0),
                stats.get("running").unwrap_or(&0),
                stats.get("stopped").unwrap_or(&0)
            );

            for server_info in servers {
                result.push_str(&server_info);
                result.push_str("\n\n");
            }

            Ok(result)
        } else {
            // Spezifischen Server anzeigen
            let server_id = args[0];
            let status = manager.get_server_status(server_id).await?;
            Ok(status)
        }
    }

    /// Server auflisten
    async fn handle_list(&self, manager: &ServerManager) -> Result<String> {
        self.handle_status(manager, &["--all"]).await
    }

    /// Server-Logs anzeigen
    async fn handle_logs(&self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(AppError::Validation(
                "Usage: server logs <server-id>".to_string(),
            ));
        }

        let _server_id = args[0];

        // TODO: Implementiere Log-Anzeige
        Ok("📋 Server Logs (TODO: Implement)\n\n\
    • Real-time logs will be shown here\n\
    • Use Ctrl+C to exit log view\n\
    • Add --follow flag for live updates"
            .to_string())
    }

    /// Zeigt Hilfe für create Befehl
    fn show_create_help(&self) -> String {
        "🏗️  Server Create Command\n\n\
    Usage: server create [OPTIONS]\n\n\
    Options:\n\
    • --port, -p <PORT>     Specify port (default: auto-find)\n\
    • --mode, -m <MODE>     Set mode: dev|prod (default: dev)\n\
    • --help, -h            Show this help\n\n\
    Examples:\n\
    • server create                    # Auto-port, dev mode\n\
    • server create --port 8080        # Specific port\n\
    • server create --mode prod        # Production mode\n\
    • server create -p 3000 -m dev     # Port + mode\n\n\
    Modes:\n\
    • dev:  Hot-reloading, CORS, debug logs\n\
    • prod: Optimized, TLS support, minimal logs"
            .to_string()
    }

    /// Zeigt allgemeine Hilfe
    fn show_help(&self) -> String {
        "🖥️  Server Management Commands\n\n\
    Usage: server <command> [OPTIONS]\n\n\
    Commands:\n\
    • create                Create new server instance\n\
    • start <id>            Start server\n\
    • stop <id>             Stop server\n\
    • delete <id>           Delete server (must be stopped)\n\
    • status [id|--all]     Show server status\n\
    • list                  List all servers\n\
    • logs <id>             Show server logs\n\n\
    Examples:\n\
    • server create --port 8080 --mode dev\n\
    • server start ABC12345\n\
    • server status --all\n\
    • server stop ABC12345\n\n\
    💡 Server IDs are shown in status output (first 8 characters)"
            .to_string()
    }
}

impl Default for ServerCommand {
    fn default() -> Self {
        Self
    }
}

// TODO: Implement Clone for ServerManager
impl Clone for ServerManager {
    fn clone(&self) -> Self {
        // Für jetzt einfache Implementierung
        // In echter Anwendung würden wir shared state verwenden
        Self {
            servers: Arc::clone(&self.servers),
            config_file: self.config_file.clone(),
        }
    }
}
