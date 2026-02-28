use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerInfo};

#[derive(Debug, Default)]
pub struct ServerManager {
    ctx: ServerContext,
}

impl ServerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_server_info(&self, identifier: &str) -> Result<ServerInfo> {
        let servers = read_lock(&self.ctx.servers, "servers")?;
        let server = crate::server::utils::validation::find_server(&servers, identifier)?;
        Ok(server.clone())
    }

    pub fn get_context(&self) -> &ServerContext {
        &self.ctx
    }
}
