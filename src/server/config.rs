pub struct ServerConfig {
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub shutdown_timeout_secs: u64,
    pub startup_delay_ms: u64,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port_range_start: 8080,
            port_range_end: 8180,
            shutdown_timeout_secs: 5,
            startup_delay_ms: 500,
            workers: 1,
        }
    }
}

pub const SERVER_VERSION: &str = "1.0.0";
pub const SERVER_NAME: &str = "Rush Sync Server";
