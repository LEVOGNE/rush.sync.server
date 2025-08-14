// =====================================================
// FILE: src/server/config.rs - SERVER CONFIGURATION
// =====================================================

use crate::core::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server-Modus: Dev mit Hot-Reloading, Prod optimiert
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMode {
    Dev,  // Hot-Reloading, Debug-Logs, CORS offen
    Prod, // Optimiert, TLS, minimale Logs
}

impl Default for ServerMode {
    fn default() -> Self {
        Self::Dev
    }
}

impl std::fmt::Display for ServerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dev => write!(f, "development"),
            Self::Prod => write!(f, "production"),
        }
    }
}

/// Server-spezifische Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub mode: ServerMode,
    pub port: u16,
    pub host: String,
    pub static_dir: PathBuf,
    pub hot_reload: bool,
    pub cors_enabled: bool,
    pub debug_logs: bool,

    // Dev-spezifische Einstellungen
    pub dev: DevConfig,

    // Prod-spezifische Einstellungen
    pub prod: ProdConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    pub auto_compile_scss: bool,
    pub watch_files: Vec<String>,
    pub reload_delay_ms: u64,
    pub show_debug_routes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProdConfig {
    pub enable_tls: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub compression: bool,
    pub cache_static_files: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            mode: ServerMode::Dev,
            port: 8080,
            host: "127.0.0.1".to_string(),
            static_dir: PathBuf::from("static"),
            hot_reload: true,
            cors_enabled: true,
            debug_logs: true,

            dev: DevConfig {
                auto_compile_scss: true,
                watch_files: vec![
                    "static/**/*.html".to_string(),
                    "static/**/*.css".to_string(),
                    "static/**/*.scss".to_string(),
                    "static/**/*.js".to_string(),
                ],
                reload_delay_ms: 100,
                show_debug_routes: true,
            },

            prod: ProdConfig {
                enable_tls: false,
                cert_path: None,
                key_path: None,
                compression: true,
                cache_static_files: true,
            },
        }
    }
}

impl ServerConfig {
    /// Erstellt Konfiguration für spezifischen Modus
    pub fn for_mode(mode: ServerMode, port: u16) -> Self {
        let (hot_reload, cors_enabled, debug_logs) = match mode {
            ServerMode::Dev => (true, true, true),
            ServerMode::Prod => (false, false, false),
        };

        Self {
            mode,
            port,
            hot_reload,
            cors_enabled,
            debug_logs,
            ..Default::default()
        }
    }

    /// Bind-Adresse für Actix-Web
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Prüft ob Konfiguration gültig ist
    pub fn validate(&self) -> Result<()> {
        if self.port < 1024 && self.host != "127.0.0.1" {
            return Err(AppError::Validation(
                "Ports < 1024 require root privileges".to_string(),
            ));
        }

        if self.mode == ServerMode::Prod
            && self.prod.enable_tls
            && (self.prod.cert_path.is_none() || self.prod.key_path.is_none())
        {
            return Err(AppError::Validation(
                "TLS enabled but cert/key paths missing".to_string(),
            ));
        }

        Ok(())
    }

    /// User-friendly Beschreibung
    pub fn description(&self) -> String {
        format!(
            "📊 Server Config\n   Mode: {}\n   Address: {}\n   Hot-Reload: {}\n   CORS: {}\n   Debug: {}",
            self.mode,
            self.bind_address(),
            self.hot_reload,
            self.cors_enabled,
            self.debug_logs
        )
    }
}
