// ## FILE: src/server/config.rs - DEDUPLIZIERT
use crate::core::config::Config;

pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SERVER_NAME: &str = env!("CARGO_PKG_NAME");

pub fn get_server_version() -> &'static str {
    SERVER_VERSION
}
pub fn get_server_name() -> &'static str {
    SERVER_NAME
}

pub fn get_server_config(config: &Config) -> &crate::core::config::ServerConfig {
    &config.server
}

pub fn get_logging_config(config: &Config) -> &crate::core::config::LoggingConfig {
    &config.logging
}
