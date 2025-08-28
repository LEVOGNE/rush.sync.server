use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Default)]
pub struct ListCommand;

impl ListCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ListCommand {
    fn name(&self) -> &'static str {
        "list"
    }
    fn description(&self) -> &'static str {
        "List all web servers (persistent)"
    }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        cmd == "list" || cmd == "list servers" || cmd == "list server"
    }

    fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        let ctx = crate::server::shared::get_shared_context();
        Ok(self.list_servers(ctx))
    }

    fn execute_async<'a>(
        &'a self,
        _args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move { self.execute_sync(_args) })
    }

    fn supports_async(&self) -> bool {
        true
    }
    fn priority(&self) -> u8 {
        60
    }
}

impl ListCommand {
    fn list_servers(&self, ctx: &ServerContext) -> String {
        let registry = crate::server::shared::get_persistent_registry();
        let servers = ctx.servers.read().unwrap();

        if servers.is_empty() {
            return format!(
                "Keine Server erstellt. Verwende 'create'\nRegistry: {}",
                registry.get_file_path().display() // FIX: get_file_path() verwenden
            );
        }

        let mut result = String::from("Server Liste (Persistent):\n");
        let mut server_list: Vec<_> = servers.values().collect();
        server_list.sort_by(|a, b| a.created_timestamp.cmp(&b.created_timestamp));

        for (i, server) in server_list.iter().enumerate() {
            let status_icon = match server.status {
                ServerStatus::Running => "Running",
                ServerStatus::Stopped => "Stopped",
                ServerStatus::Failed => "Failed",
            };

            result.push_str(&format!(
                "  {}. {} - {} (Port: {}) [{}]\n",
                i + 1,
                server.name,
                &server.id[0..8],
                server.port,
                status_icon
            ));
        }

        result.push_str(&format!(
            "\nRegistry: {}",
            registry.get_file_path().display() // FIX: get_file_path() verwenden
        ));

        result
    }
}
