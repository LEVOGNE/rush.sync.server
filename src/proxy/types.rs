use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub port: u16,              // HTTP Proxy Port (3000)
    pub https_port_offset: u16, // NEU: HTTPS Offset (443)
    pub bind_address: String,
    pub health_check_interval: u64,
    pub timeout_ms: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 3000,             // HTTP Proxy
            https_port_offset: 443, // HTTPS = 3000 + 443 = 3443
            bind_address: "127.0.0.1".to_string(),
            health_check_interval: 30,
            timeout_ms: 5000,
        }
    }
}

// NEU: TOML-spezifische Struktur für Serialisierung
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyConfigToml {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub health_check_interval: u64,
    pub timeout_ms: u64,
    pub https_port_offset: u16,
}

// FEHLEND: Default für ProxyConfigToml
impl Default for ProxyConfigToml {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 3000,
            https_port_offset: 443,
            bind_address: "127.0.0.1".to_string(),
            health_check_interval: 30,
            timeout_ms: 5000,
        }
    }
}

impl From<ProxyConfig> for ProxyConfigToml {
    fn from(config: ProxyConfig) -> Self {
        Self {
            enabled: config.enabled,
            port: config.port,
            https_port_offset: config.https_port_offset,
            bind_address: config.bind_address,
            health_check_interval: config.health_check_interval,
            timeout_ms: config.timeout_ms,
        }
    }
}

impl From<ProxyConfigToml> for ProxyConfig {
    fn from(config: ProxyConfigToml) -> Self {
        Self {
            enabled: config.enabled,
            port: config.port,
            https_port_offset: config.https_port_offset,
            bind_address: config.bind_address,
            health_check_interval: config.health_check_interval,
            timeout_ms: config.timeout_ms,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyTarget {
    pub name: String,
    pub port: u16,
    pub healthy: bool,
    pub last_check: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct ProxyRoute {
    pub subdomain: String,
    pub target_port: u16,
    pub server_id: String,
}

pub type RouteMap = HashMap<String, ProxyRoute>;
