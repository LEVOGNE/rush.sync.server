// src/server/persistence.rs
use crate::core::prelude::*;
use crate::server::types::{ServerInfo, ServerStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistentServerInfo {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub status: ServerStatus,
    pub created_at: String,
    pub created_timestamp: u64,
    pub auto_start: bool,
    pub last_started: Option<String>,
    pub start_count: u32,
}

impl From<ServerInfo> for PersistentServerInfo {
    fn from(info: ServerInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            port: info.port,
            status: info.status,
            created_at: info.created_at,
            created_timestamp: info.created_timestamp,
            auto_start: false,
            last_started: None,
            start_count: 0,
        }
    }
}

impl From<PersistentServerInfo> for ServerInfo {
    fn from(info: PersistentServerInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            port: info.port,
            status: info.status,
            created_at: info.created_at,
            created_timestamp: info.created_timestamp,
        }
    }
}

pub struct ServerRegistry {
    file_path: PathBuf,
}

impl ServerRegistry {
    pub fn new() -> Result<Self> {
        let base_dir = crate::core::helpers::get_base_dir()?;

        let file_path = base_dir.join(".rss").join("servers.list");

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }

        Ok(Self { file_path })
    }

    /// Fallback constructor that never fails — uses temp dir if base_dir is unavailable
    pub fn with_fallback() -> Self {
        match Self::new() {
            Ok(registry) => registry,
            Err(_) => {
                let path = std::env::temp_dir().join(".rss").join("servers.list");
                let _ = std::fs::create_dir_all(path.parent().unwrap_or(&path));
                Self { file_path: path }
            }
        }
    }

    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub async fn load_servers(&self) -> Result<HashMap<String, PersistentServerInfo>> {
        if !self.file_path.exists() {
            return Ok(HashMap::new());
        }

        let content = tokio::fs::read_to_string(&self.file_path)
            .await
            .map_err(AppError::Io)?;
        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }

        let servers: Vec<PersistentServerInfo> = serde_json::from_str(&content)
            .map_err(|e| AppError::Validation(format!("Failed to parse server registry: {}", e)))?;

        Ok(servers.into_iter().map(|s| (s.id.clone(), s)).collect())
    }

    pub async fn save_servers(
        &self,
        servers: &HashMap<String, PersistentServerInfo>,
    ) -> Result<()> {
        let mut server_list: Vec<PersistentServerInfo> = servers.values().cloned().collect();
        server_list.sort_by_key(|s| s.created_timestamp);

        let content = serde_json::to_string_pretty(&server_list)
            .map_err(|e| AppError::Validation(format!("Failed to serialize servers: {}", e)))?;

        let temp_path = self.file_path.with_extension("tmp");
        tokio::fs::write(&temp_path, content)
            .await
            .map_err(AppError::Io)?;
        tokio::fs::rename(&temp_path, &self.file_path)
            .await
            .map_err(AppError::Io)?;

        Ok(())
    }

    // Generic update helper to reduce boilerplate
    async fn update_server(
        &self,
        server_id: &str,
        update_fn: impl Fn(&mut PersistentServerInfo),
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        let mut servers = self.load_servers().await?;
        if let Some(server) = servers.get_mut(server_id) {
            update_fn(server);
            self.save_servers(&servers).await?;
        }
        Ok(servers)
    }

    pub async fn update_server_status(
        &self,
        server_id: &str,
        status: ServerStatus,
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        self.update_server(server_id, |server| {
            server.status = status;
            if status == ServerStatus::Running {
                server.last_started =
                    Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                server.start_count += 1;
            }
        })
        .await
    }

    pub async fn set_auto_start(
        &self,
        server_id: &str,
        auto_start: bool,
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        self.update_server(server_id, |server| {
            server.auto_start = auto_start;
        })
        .await
    }

    pub async fn add_server(
        &self,
        server_info: ServerInfo,
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        let mut servers = self.load_servers().await?;
        let persistent_info = PersistentServerInfo::from(server_info);
        servers.insert(persistent_info.id.clone(), persistent_info);
        self.save_servers(&servers).await?;
        Ok(servers)
    }

    pub async fn remove_server(
        &self,
        server_id: &str,
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        let mut servers = self.load_servers().await?;
        servers.remove(server_id);
        self.save_servers(&servers).await?;
        Ok(servers)
    }

    pub async fn cleanup_servers(
        &self,
        cleanup_type: CleanupType,
    ) -> Result<(HashMap<String, PersistentServerInfo>, usize)> {
        let mut servers = self.load_servers().await?;
        let initial_count = servers.len();

        servers.retain(|_, s| match cleanup_type {
            CleanupType::Stopped => s.status != ServerStatus::Stopped,
            CleanupType::Failed => s.status != ServerStatus::Failed,
            CleanupType::All => s.status == ServerStatus::Running,
        });

        let removed_count = initial_count - servers.len();
        if removed_count > 0 {
            self.save_servers(&servers).await?;
        }

        Ok((servers, removed_count))
    }

    pub fn get_auto_start_servers(
        &self,
        servers: &HashMap<String, PersistentServerInfo>,
    ) -> Vec<PersistentServerInfo> {
        servers
            .values()
            .filter(|s| s.auto_start && s.status != ServerStatus::Failed)
            .cloned()
            .collect()
    }

    // Directory cleanup utilities
    pub async fn cleanup_server_directory(&self, server_name: &str, port: u16) -> Result<()> {
        let base_dir = crate::core::helpers::get_base_dir()?;

        let server_dir = base_dir
            .join("www")
            .join(format!("{}-[{}]", server_name, port));

        if server_dir.exists() {
            std::fs::remove_dir_all(&server_dir).map_err(AppError::Io)?;
            log::info!("Removed server directory: {:?}", server_dir);
        }
        Ok(())
    }

    pub fn list_www_directories(&self) -> Result<Vec<PathBuf>> {
        let base_dir = crate::core::helpers::get_base_dir()?;

        let www_dir = base_dir.join("www");
        if !www_dir.exists() {
            return Ok(vec![]);
        }

        let mut directories = Vec::new();
        for entry in std::fs::read_dir(&www_dir).map_err(AppError::Io)? {
            let entry = entry.map_err(AppError::Io)?;
            if entry.file_type().map_err(AppError::Io)?.is_dir() {
                directories.push(entry.path());
            }
        }
        directories.sort();
        Ok(directories)
    }
}

#[derive(Debug)]
pub enum CleanupType {
    Stopped,
    Failed,
    All,
}
