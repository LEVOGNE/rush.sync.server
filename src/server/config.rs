use crate::core::config::Config;

pub fn get_server_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_server_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SERVER_NAME: &str = env!("CARGO_PKG_NAME");

pub fn get_server_config_from_main(config: &Config) -> &crate::core::config::ServerConfig {
    &config.server
}

pub fn get_logging_config_from_main(config: &Config) -> &crate::core::config::LoggingConfig {
    &config.logging
}
