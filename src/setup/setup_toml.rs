// Enhanced src/setup/setup_toml.rs
use crate::core::prelude::*;
use std::path::PathBuf;
use tokio::fs;

// Enhanced DEFAULT_CONFIG with server and logging sections
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
# SERVER MANAGEMENT CONFIGURATION
# =====================================================
[server]
port_range_start = 8080      # Starting port for auto-allocation
port_range_end = 8180        # Maximum port for auto-allocation
max_concurrent = 10          # Maximum simultaneous servers
shutdown_timeout = 5         # Graceful shutdown timeout (seconds)
startup_delay_ms = 500       # Delay after server creation (milliseconds)
workers = 1                  # Actix workers per server

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
    let exe_path = std::env::current_exe().map_err(AppError::Io)?;
    let base_dir = exe_path
        .parent()
        .ok_or_else(|| AppError::Validation(get_translation("system.config.dir_error", &[])))?;

    let config_dir = base_dir.join(".rss");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .await
            .map_err(AppError::Io)?;
    }

    let config_path = config_dir.join("rush.toml");
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
            paths.push(base_dir.join(".rss/rush.toml"));
            paths.push(base_dir.join("rush.toml"));
            paths.push(base_dir.join("config/rush.toml"));
        }
    }
    #[cfg(debug_assertions)]
    {
        paths.push(PathBuf::from("rush.toml"));
        paths.push(PathBuf::from("src/rush.toml"));
    }
    paths
}
