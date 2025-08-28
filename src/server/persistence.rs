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
        let exe_path = std::env::current_exe().map_err(AppError::Io)?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Cannot determine executable directory".to_string())
        })?;

        let file_path = base_dir.join(".rss").join("servers.list");

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }

        Ok(Self { file_path })
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
        server_list.sort_by(|a, b| a.created_timestamp.cmp(&b.created_timestamp));

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

    pub async fn add_server(
        &self,
        mut servers: HashMap<String, PersistentServerInfo>,
        server_info: ServerInfo,
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        let persistent_info = PersistentServerInfo::from(server_info);
        servers.insert(persistent_info.id.clone(), persistent_info);
        self.save_servers(&servers).await?;
        Ok(servers)
    }

    pub async fn remove_server(
        &self,
        mut servers: HashMap<String, PersistentServerInfo>,
        server_id: &str,
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        servers.remove(server_id);
        self.save_servers(&servers).await?;
        Ok(servers)
    }

    pub async fn update_server_status(
        &self,
        mut servers: HashMap<String, PersistentServerInfo>,
        server_id: &str,
        status: ServerStatus,
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        if let Some(server) = servers.get_mut(server_id) {
            server.status = status;
            if status == ServerStatus::Running {
                server.last_started =
                    Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                server.start_count += 1;
            }
        }
        self.save_servers(&servers).await?;
        Ok(servers)
    }

    pub async fn cleanup_servers(
        &self,
        mut servers: HashMap<String, PersistentServerInfo>,
        cleanup_type: CleanupType,
    ) -> Result<(HashMap<String, PersistentServerInfo>, usize)> {
        let initial_count = servers.len();

        match cleanup_type {
            CleanupType::Stopped => servers.retain(|_, s| s.status != ServerStatus::Stopped),
            CleanupType::Failed => servers.retain(|_, s| s.status != ServerStatus::Failed),
            CleanupType::All => servers.retain(|_, s| s.status == ServerStatus::Running),
        }

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

    pub async fn set_auto_start(
        &self,
        mut servers: HashMap<String, PersistentServerInfo>,
        server_id: &str,
        auto_start: bool,
    ) -> Result<HashMap<String, PersistentServerInfo>> {
        if let Some(server) = servers.get_mut(server_id) {
            server.auto_start = auto_start;
            self.save_servers(&servers).await?;
        }
        Ok(servers)
    }
}

#[derive(Debug)]
pub enum CleanupType {
    Stopped,
    Failed,
    All,
}
