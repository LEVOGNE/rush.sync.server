// =====================================================
// FILE: src/server/manager.rs - SERVER MANAGER
// =====================================================

use crate::core::prelude::*;
use crate::server::{ServerInfo, ServerInstance, ServerMode, ServerStatus};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

/// Zentrale Verwaltung aller Server-Instanzen
pub struct ServerManager {
    pub(crate) servers: Arc<RwLock<HashMap<String, ServerInstance>>>,
    pub(crate) config_file: std::path::PathBuf,
}

impl ServerManager {
    /// Erstellt neuen Server-Manager
    pub fn new() -> Self {
        let config_file = std::path::PathBuf::from("config/servers.json");

        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            config_file,
        }
    }

    /// Lädt gespeicherte Server-Konfigurationen
    pub async fn load_servers(&self) -> Result<()> {
        if !self.config_file.exists() {
            log::info!("📁 No existing server config, starting fresh");
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.config_file)
            .await
            .map_err(AppError::Io)?;

        let server_infos: Vec<ServerInfo> = serde_json::from_str(&content)
            .map_err(|e| AppError::Validation(format!("Invalid server config: {}", e)))?;

        let mut servers = self.servers.write().await;
        for info in server_infos {
            let server = ServerInstance::from_info(info);
            servers.insert(server.get_server_id(), server);
        }

        log::info!("📊 Loaded {} servers from config", servers.len());
        Ok(())
    }

    /// Speichert Server-Konfigurationen
    pub async fn save_servers(&self) -> Result<()> {
        // Config-Verzeichnis erstellen
        if let Some(parent) = self.config_file.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::Io)?;
        }

        let servers = self.servers.read().await;
        let server_infos: Vec<ServerInfo> = servers
            .values()
            .map(|server| {
                let info = server.info.lock().unwrap_or_else(|poisoned| {
                    log::warn!("Recovered from poisoned mutex");
                    poisoned.into_inner()
                });
                info.clone()
            })
            .collect();

        let content = serde_json::to_string_pretty(&server_infos)
            .map_err(|e| AppError::Validation(format!("Failed to serialize servers: {}", e)))?;

        tokio::fs::write(&self.config_file, content)
            .await
            .map_err(AppError::Io)?;

        log::debug!("💾 Saved {} servers to config", server_infos.len());
        Ok(())
    }

    /// Erstellt einen neuen Server
    pub async fn create_server(&self, port: u16, mode: ServerMode) -> Result<String> {
        // Prüfe ob Port bereits verwendet wird
        let servers = self.servers.read().await;
        for server in servers.values() {
            let info = server.info.lock().unwrap_or_else(|poisoned| {
                log::warn!("Recovered from poisoned mutex");
                poisoned.into_inner()
            });
            if info.port == port {
                return Err(AppError::Validation(format!(
                    "Port {} already in use",
                    port
                )));
            }
        }
        drop(servers);

        // Neuen Server erstellen
        let server = ServerInstance::new(port, mode);
        let server_id = server.get_server_id();

        // Server zur Verwaltung hinzufügen
        let mut servers = self.servers.write().await;
        servers.insert(server_id.clone(), server);
        drop(servers);

        // Konfiguration speichern
        self.save_servers().await?;

        log::info!(
            "✅ Created server {} on port {} ({})",
            server_id,
            port,
            mode
        );
        Ok(server_id)
    }

    /// Startet einen Server
    /// Startet einen Server
    pub async fn start_server(&self, server_id: &str) -> Result<()> {
        // Direkt Write-Lock holen
        let mut servers = self.servers.write().await;

        // Server direkt mutieren, KEIN Clone!
        let server = servers
            .get_mut(server_id)
            .ok_or_else(|| AppError::Validation(format!("Server {} not found", server_id)))?;

        // Status prüfen
        match server.get_status() {
            ServerStatus::Running => {
                return Err(AppError::Validation(format!(
                    "Server {} already running",
                    server_id
                )));
            }
            ServerStatus::Starting => {
                return Err(AppError::Validation(format!(
                    "Server {} already starting",
                    server_id
                )));
            }
            _ => {}
        }

        // Server direkt starten (kein Clone!)
        server.start().await?;

        // Servers lock freigeben vor save
        drop(servers);

        // Speichern
        self.save_servers().await?;

        Ok(())
    }

    /// Stoppt einen Server
    pub async fn stop_server(&self, server_id: &str) -> Result<()> {
        // Write-Lock direkt
        let mut servers = self.servers.write().await;

        let server = servers
            .get_mut(server_id)
            .ok_or_else(|| AppError::Validation(format!("Server {} not found", server_id)))?;

        match server.get_status() {
            ServerStatus::Stopped => {
                return Err(AppError::Validation(format!(
                    "Server {} already stopped",
                    server_id
                )));
            }
            ServerStatus::Stopping => {
                return Err(AppError::Validation(format!(
                    "Server {} already stopping",
                    server_id
                )));
            }
            _ => {}
        }

        // Direkt stoppen
        server.stop().await?;

        drop(servers);
        self.save_servers().await?;

        Ok(())
    }

    /// Löscht einen Server
    pub async fn delete_server(&self, server_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;

        let server = servers
            .get(server_id)
            .ok_or_else(|| AppError::Validation(format!("Server {} not found", server_id)))?;

        // Server muss gestoppt sein
        match server.get_status() {
            ServerStatus::Running | ServerStatus::Starting => {
                return Err(AppError::Validation(format!(
                    "Server {} must be stopped before deletion",
                    server_id
                )));
            }
            _ => {}
        }

        // Working Directory löschen
        let working_dir = {
            let info = server.info.lock().unwrap_or_else(|poisoned| {
                log::warn!("Recovered from poisoned mutex");
                poisoned.into_inner()
            });
            info.working_dir.clone()
        };

        if working_dir.exists() {
            tokio::fs::remove_dir_all(&working_dir)
                .await
                .map_err(AppError::Io)?;
            log::debug!("🗑️  Deleted working directory: {}", working_dir.display());
        }

        // Server aus Verwaltung entfernen
        servers.remove(server_id);
        drop(servers);

        self.save_servers().await?;

        log::info!("🗑️  Deleted server {}", server_id);
        Ok(())
    }

    /// Liste aller Server
    pub async fn list_servers(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.values().map(|server| server.debug_info()).collect()
    }

    /// Server-Status anzeigen
    pub async fn get_server_status(&self, server_id: &str) -> Result<String> {
        let servers = self.servers.read().await;

        let server = servers
            .get(server_id)
            .ok_or_else(|| AppError::Validation(format!("Server {} not found", server_id)))?;

        Ok(server.debug_info())
    }

    /// Alle Server stoppen (für Shutdown)
    pub async fn stop_all_servers(&self) -> Result<()> {
        let servers = self.servers.read().await;
        let server_ids: Vec<String> = servers.keys().cloned().collect();
        drop(servers);

        for server_id in server_ids {
            if let Err(e) = self.stop_server(&server_id).await {
                log::warn!("Failed to stop server {}: {}", server_id, e);
            }
        }

        Ok(())
    }

    /// Suche freien Port
    pub async fn find_free_port(&self, start_port: u16) -> u16 {
        let servers = self.servers.read().await;
        let used_ports: std::collections::HashSet<u16> = servers
            .values()
            .map(|server| {
                let info = server.info.lock().unwrap_or_else(|poisoned| {
                    log::warn!("Recovered from poisoned mutex");
                    poisoned.into_inner()
                });
                info.port
            })
            .collect();

        for port in start_port..=65535 {
            if !used_ports.contains(&port) && self.is_port_available(port).await {
                return port;
            }
        }

        8080 // Fallback
    }

    /// Prüft ob Port verfügbar ist
    async fn is_port_available(&self, port: u16) -> bool {
        use std::net::TcpListener;

        TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
    }

    /// Statistiken
    pub async fn get_stats(&self) -> HashMap<String, usize> {
        let servers = self.servers.read().await;
        let mut stats = HashMap::new();

        stats.insert("total".to_string(), servers.len());

        let mut running = 0;
        let mut stopped = 0;

        for server in servers.values() {
            match server.get_status() {
                ServerStatus::Running => running += 1,
                ServerStatus::Stopped => stopped += 1,
                _ => {}
            }
        }

        stats.insert("running".to_string(), running);
        stats.insert("stopped".to_string(), stopped);

        stats
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerInstance {
    /// Erstellt ServerInstance aus bestehender ServerInfo
    pub fn from_info(info: ServerInfo) -> Self {
        let config = crate::server::config::ServerConfig::for_mode(info.mode, info.port);

        Self {
            info: Arc::new(Mutex::new(info)),
            config,
            shutdown_tx: None,
            server_handle: None,
            file_watcher: None,
        }
    }
}
