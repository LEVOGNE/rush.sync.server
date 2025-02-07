use crate::core::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

pub struct ConfigHandler {
    config_path: PathBuf,
    settings: HashMap<String, String>,
}

impl ConfigHandler {
    pub async fn new() -> Result<Self> {
        let config_path = Self::get_config_path().await?;
        let settings = Self::load_settings(&config_path).await?;

        Ok(Self {
            config_path,
            settings,
        })
    }

    async fn get_config_path() -> Result<PathBuf> {
        let exe_path = std::env::current_exe().map_err(|e| AppError::Io(e))?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Konnte Programmverzeichnis nicht ermitteln".to_string())
        })?;

        let config_dir = base_dir.join(".rss");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .await
                .map_err(|e| AppError::Io(e))?;
            log::debug!("Konfigurationsverzeichnis erstellt: {:?}", config_dir);
        }

        Ok(config_dir.join("rush.config"))
    }

    async fn load_settings(config_path: &PathBuf) -> Result<HashMap<String, String>> {
        let mut settings = HashMap::new();

        if config_path.exists() {
            let content = fs::read_to_string(config_path)
                .await
                .map_err(|e| AppError::Io(e))?;

            for line in content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    settings.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        // Setze Standardwerte falls nicht vorhanden
        if !settings.contains_key("lang") {
            settings.insert("lang".to_string(), "de".to_string());
        }

        Ok(settings)
    }

    pub async fn save_settings(&self) -> Result<()> {
        let mut content = String::new();
        for (key, value) in &self.settings {
            content.push_str(&format!("{}={}\n", key, value));
        }

        fs::write(&self.config_path, content)
            .await
            .map_err(|e| AppError::Io(e))?;

        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.settings.get(key).cloned()
    }

    pub async fn set_setting(&mut self, key: String, value: String) -> Result<()> {
        self.settings.insert(key, value);
        self.save_settings().await
    }
}
