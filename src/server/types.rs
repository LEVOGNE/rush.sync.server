// ## FILE: src/server/types.rs - KOMPLETT KORRIGIERT
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Running,
    Failed,
}

impl std::fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Stopped => "STOPPED",
                Self::Running => "RUNNING",
                Self::Failed => "FAILED",
            }
        )
    }
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

// WICHTIG: ServerData-Struktur hinzugefügt
#[derive(Debug, Clone)]
pub struct ServerData {
    pub id: String,
    pub port: u16,
    pub name: String,
}

pub type ServerMap = Arc<RwLock<HashMap<String, ServerInfo>>>;
pub type ServerHandles = Arc<RwLock<HashMap<String, actix_web::dev::ServerHandle>>>;

#[derive(Debug, Clone, Default)]
pub struct ServerContext {
    pub servers: ServerMap,
    pub handles: ServerHandles,
}
