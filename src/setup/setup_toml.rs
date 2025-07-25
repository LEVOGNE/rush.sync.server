use crate::core::prelude::*;
use std::path::PathBuf;
use tokio::fs;

const DEFAULT_CONFIG: &str = r#"[general]
max_messages = 100
typewriter_delay = 30
input_max_length = 100
max_history = 30
poll_rate = 16

[theme]
input_text = "Black"
input_bg = "White"
cursor = "Black"
output_text = "DarkGray"
output_bg = "Black"

[prompt]
text = "/// "
color = "Black"
"#;

pub async fn ensure_config_exists() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().map_err(AppError::Io)?;
    let base_dir = exe_path.parent().ok_or_else(|| {
        AppError::Validation("Konnte Programmverzeichnis nicht ermitteln".to_string())
    })?;

    let config_dir = base_dir.join(".rss");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .await
            .map_err(AppError::Io)?;
        log::debug!("Konfig-Verzeichnis erstellt: {}", config_dir.display());
    }

    let config_path = config_dir.join("rush.toml");
    if !config_path.exists() {
        fs::write(&config_path, DEFAULT_CONFIG)
            .await
            .map_err(AppError::Io)?;
        log::info!("Default rush.toml erstellt: {}", config_path.display());
    }

    Ok(config_path)
}

pub fn get_config_paths() -> Vec<PathBuf> {
    // unverändert
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
