use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub status: ServerStatus,
    pub created_at: String,
    pub created_timestamp: u64,
}

impl Default for ServerInfo {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: String::new(),
            name: String::new(),
            port: 0,
            status: ServerStatus::Stopped,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            created_timestamp: now,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Running,
    Failed,
}

impl std::fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerStatus::Stopped => write!(f, "STOPPED"),
            ServerStatus::Running => write!(f, "RUNNING"),
            ServerStatus::Failed => write!(f, "FAILED"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerData {
    pub id: String,
    pub port: u16,
    pub name: String,
}

pub type ServerMap = Arc<RwLock<HashMap<String, ServerInfo>>>;
pub type ServerHandles = Arc<RwLock<HashMap<String, actix_web::dev::ServerHandle>>>;

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub servers: ServerMap,
    pub handles: ServerHandles,
}

impl Default for ServerContext {
    fn default() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
// ## END ##

// ## FILE: src/server/config.rs
// ## BEGIN ##
use crate::core::config::Config;

pub fn get_server_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_server_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SERVER_NAME: &str = env!("CARGO_PKG_NAME");

pub fn get_server_config_from_main(config: &Config) -> &crate::core::config::ServerConfig {
    &config.server
}

pub fn get_logging_config_from_main(config: &Config) -> &crate::core::config::LoggingConfig {
    &config.logging
}
