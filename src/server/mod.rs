// =====================================================
// FILE: src/server/mod.rs - ACTIX-WEB SERVER MODULE
// =====================================================

pub mod config;
pub mod instance;
pub mod manager;
pub mod middleware;
pub mod routes;

use chrono::{DateTime, Utc};
pub use config::{ServerConfig, ServerMode};
pub use instance::ServerInstance;
pub use manager::ServerManager;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Server-Status für Tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

/// Server-Information für Verwaltung
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerInfo {
    pub id: String,
    pub hash: String,
    pub port: u16,
    pub mode: ServerMode,
    pub status: ServerStatus,
    pub working_dir: std::path::PathBuf,
    pub created_at: DateTime<Utc>,
    pub last_modified: Option<DateTime<Utc>>,
}

impl ServerInfo {
    /// Erstellt neue Server-Info mit eindeutiger ID
    pub fn new(port: u16, mode: ServerMode) -> Self {
        let id = Uuid::new_v4().to_string();
        let hash = format!("{:x}", Sha256::digest(id.as_bytes()))[..8].to_string();

        // working_dir VOR dem Struct bauen (nutzt nur &hash, kein Move)
        let working_dir = std::path::PathBuf::from(format!("servers/server_{hash}"));

        Self {
            id,   // id kann direkt moved werden
            hash, // hash wird hier genau einmal moved
            port,
            mode,
            status: ServerStatus::Stopped,
            working_dir, // bereits gebaut, eigener Move
            created_at: Utc::now(),
            last_modified: None,
        }
    }

    /// Debug-Info für CLI-Ausgabe
    pub fn debug_info(&self) -> String {
        format!(
            "🖥️  Server {} ({})\n   Port: {}\n   Mode: {:?}\n   Status: {:?}\n   Dir: {}",
            self.id[..8].to_uppercase(),
            self.hash,
            self.port,
            self.mode,
            self.status,
            self.working_dir.display()
        )
    }
}
