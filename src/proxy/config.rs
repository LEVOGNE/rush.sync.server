use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub port: u16,
    pub enable_tls: bool,
    pub cert_dir: String,
    pub max_concurrent: usize,
    pub timeout_seconds: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8000,
            enable_tls: true,
            cert_dir: ".rss/certs".to_string(),
            max_concurrent: 50,
            timeout_seconds: 30,
        }
    }
}
