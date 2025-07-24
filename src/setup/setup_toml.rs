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
    // Hole den Pfad der ausführbaren Datei
    let exe_path = std::env::current_exe().map_err(|e| AppError::Io(e))?;
    let base_dir = exe_path.parent().ok_or_else(|| {
        AppError::Validation("Konnte Programmverzeichnis nicht ermitteln".to_string())
    })?;

    // Erstelle .rss Verzeichnis neben der ausführbaren Datei
    let config_dir = base_dir.join(".rss");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .await
            .map_err(|e| AppError::Io(e))?;

        // Direkte Formatierung ohne Translation-Key
        let msg = format!("Konfigurationsverzeichnis erstellt: {:?}", config_dir);
        log::debug!(
            "{}",
            AppColor::from_category(ColorCategory::Info).format_message("DEBUG", &msg)
        );
    }

    // Pfad zur rush.toml im .rss Verzeichnis
    let config_path = config_dir.join("rush.toml");

    // Erstelle rush.toml falls sie nicht existiert
    if !config_path.exists() {
        fs::write(&config_path, DEFAULT_CONFIG)
            .await
            .map_err(|e| AppError::Io(e))?;

        // Direkte Formatierung ohne Translation-Key
        let msg = format!("Standard-Konfigurationsdatei erstellt: {:?}", config_path);
        log::info!(
            "{}",
            AppColor::from_category(ColorCategory::Info).format_message("INFO", &msg)
        );
    }

    Ok(config_path)
}

pub fn get_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Versuche den Executable-Pfad zu bekommen
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(base_dir) = exe_path.parent() {
            // Konfigurationsdatei im .rss Verzeichnis neben der Executable
            paths.push(base_dir.join(".rss/rush.toml"));

            // Weitere mögliche Pfade relativ zur Executable
            paths.push(base_dir.join("rush.toml"));
            paths.push(base_dir.join("config/rush.toml"));
        }
    }

    // Fallback für Entwicklungsumgebung
    #[cfg(debug_assertions)]
    {
        paths.push(PathBuf::from("rush.toml"));
        paths.push(PathBuf::from("src/rush.toml"));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_paths() {
        let paths = get_config_paths();
        assert!(
            !paths.is_empty(),
            "Konfigurationspfade sollten nicht leer sein"
        );
    }
}
