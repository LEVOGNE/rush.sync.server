// src/setup/setup_toml.rs - Cleaned and optimized
use crate::core::prelude::*;
use std::path::PathBuf;
use tokio::fs;

// Consolidated DEFAULT_CONFIG - All sections in one place
const DEFAULT_CONFIG: &str = r#"[general]
max_messages = 1000
typewriter_delay = 5
input_max_length = 100
max_history = 30
poll_rate = 16
log_level = "info"
current_theme = "dark"

[language]
current = "en"

# =====================================================
# SERVER CONFIGURATION
# =====================================================
[server]
# Port Management
port_range_start = 8000      # Starting port for auto-allocation
port_range_end = 8200        # Maximum port for auto-allocation
max_concurrent = 10          # Maximum simultaneous servers
shutdown_timeout = 5         # Graceful shutdown timeout (seconds)
startup_delay_ms = 500       # Delay after server creation (milliseconds)
workers = 1                  # Actix workers per server
auto_open_browser = true     # Automatically open browser

# HTTPS/TLS Configuration
enable_https = true          # Enable HTTPS support
https_port_offset = 1000     # HTTPS port = HTTP port + offset
cert_dir = ".rss/certs"      # Certificate storage directory
auto_cert = true             # Generate certificates automatically
cert_validity_days = 365     # Certificate validity (days)

# Production Settings
use_lets_encrypt = false     # Use Let's Encrypt (requires public domain)
production_domain = "localhost"  # Production domain name

# =====================================================
# REVERSE PROXY CONFIGURATION
# =====================================================
[proxy]
enabled = true                  # Enable integrated reverse proxy
port = 3000                     # Proxy listening port
https_port_offset = 443         # Neu hinzufügen
bind_address = "127.0.0.1"      # Proxy bind address
health_check_interval = 30      # Health check interval (seconds)
timeout_ms = 5000               # Request timeout (milliseconds)

# For production use:
# port = 80                  # Standard HTTP Port
# bind_address = "0.0.0.0"   # All interfaces (for external access)

# =====================================================
# LOGGING CONFIGURATION
# =====================================================
[logging]
max_file_size_mb = 100       # Log rotation size (100MB per file)
max_archive_files = 9        # Archive generations (9 backups)
compress_archives = true     # GZIP compression for archives
log_requests = true          # Enable request logging
log_security_alerts = true  # Enable security monitoring
log_performance = true       # Enable performance metrics

# =====================================================
# THEME DEFINITIONS
# =====================================================
[theme.dark]
output_bg = "Black"
output_text = "White"
output_cursor = "PIPE"
output_cursor_color = "White"
input_bg = "White"
input_text = "Black"
input_cursor_prefix = "/// "
input_cursor = "PIPE"
input_cursor_color = "Black"

[theme.light]
output_bg = "White"
output_text = "Black"
output_cursor = "PIPE"
output_cursor_color = "Black"
input_bg = "Black"
input_text = "White"
input_cursor_prefix = "/// "
input_cursor = "PIPE"
input_cursor_color = "White"

[theme.green]
output_bg = "Black"
output_text = "Green"
output_cursor = "BLOCK"
output_cursor_color = "Green"
input_bg = "LightGreen"
input_text = "Black"
input_cursor_prefix = "$ "
input_cursor = "BLOCK"
input_cursor_color = "Black"

[theme.blue]
output_bg = "White"
output_text = "LightBlue"
output_cursor = "UNDERSCORE"
output_cursor_color = "Blue"
input_bg = "Blue"
input_text = "White"
input_cursor_prefix = "> "
input_cursor = "UNDERSCORE"
input_cursor_color = "White"
"#;

pub async fn ensure_config_exists() -> Result<PathBuf> {
    let config_path = get_primary_config_path()?;

    // Create directory if needed
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).await.map_err(AppError::Io)?;
    }

    // Create config file if it doesn't exist
    if !config_path.exists() {
        fs::write(&config_path, DEFAULT_CONFIG)
            .await
            .map_err(AppError::Io)?;

        log::info!(
            "{}",
            get_translation(
                "system.config.file_created",
                &[&config_path.display().to_string()]
            )
        );
    }

    Ok(config_path)
}

pub fn get_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(base_dir) = exe_path.parent() {
            // Primary locations (in order of preference)
            paths.push(base_dir.join(".rss/rush.toml"));
            paths.push(base_dir.join("rush.toml"));
            paths.push(base_dir.join("config/rush.toml"));
        }
    }

    // Development fallbacks
    #[cfg(debug_assertions)]
    {
        paths.push(PathBuf::from("rush.toml"));
        paths.push(PathBuf::from("src/rush.toml"));
    }

    paths
}

fn get_primary_config_path() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().map_err(AppError::Io)?;
    let base_dir = exe_path
        .parent()
        .ok_or_else(|| AppError::Validation(get_translation("system.config.dir_error", &[])))?;

    Ok(base_dir.join(".rss/rush.toml"))
}
