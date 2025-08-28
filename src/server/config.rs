// Updated src/server/config.rs - Use Config from main config
use crate::core::config::Config;

// Remove old ServerConfig - use the one from core::config
pub fn get_server_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_server_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

// Legacy constants for backward compatibility
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SERVER_NAME: &str = env!("CARGO_PKG_NAME");

// Helper to get server config from main config
pub fn get_server_config_from_main(config: &Config) -> &crate::core::config::ServerConfig {
    &config.server
}

// Helper to get logging config from main config
pub fn get_logging_config_from_main(config: &Config) -> &crate::core::config::LoggingConfig {
    &config.logging
}
