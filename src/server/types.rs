// src/server/types.rs - CLEANED VERSION
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
    #[serde(default = "default_timestamp")]
    pub created_timestamp: u64,
}

fn default_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Shared state between all modules
pub type ServerMap = Arc<RwLock<HashMap<String, ServerInfo>>>;
pub type ServerHandles = Arc<RwLock<HashMap<String, actix_web::dev::ServerHandle>>>;

/// Context for all server operations - CLEANED
#[derive(Debug, Clone)]
pub struct ServerContext {
    pub servers: ServerMap,
    pub handles: ServerHandles,
    pub port_range_start: u16,
}

impl ServerContext {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            handles: Arc::new(RwLock::new(HashMap::new())),
            port_range_start: 8080,
        }
    }
}
