// src/server/manager.rs - MINIMAL VERSION (optional utility wrapper)
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerInfo};

#[derive(Debug)]
pub struct ServerManager {
    ctx: ServerContext,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            ctx: ServerContext::new(),
        }
    }

    // Optional: Get server info utility
    pub fn get_server_info(&self, identifier: &str) -> Result<ServerInfo> {
        let servers = self.ctx.servers.read().unwrap();
        let server = crate::server::utils::validation::find_server(&servers, identifier)?;
        Ok(server.clone())
    }

    // Optional: Access to context for complex operations
    pub fn get_context(&self) -> &ServerContext {
        &self.ctx
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}
