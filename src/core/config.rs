use crate::core::constants::{DEFAULT_BUFFER_SIZE, DEFAULT_POLL_RATE};
use crate::core::prelude::*;
use crate::ui::color::AppColor;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    general: GeneralConfig,
    theme: ThemeConfig,
    prompt: PromptConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneralConfig {
    max_messages: usize,
    typewriter_delay: u64,
    input_max_length: usize,
    max_history: usize,
    poll_rate: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThemeConfig {
    input_text: String,
    input_bg: String,
    cursor: String,
    output_text: String,
    output_bg: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptConfig {
    text: String,
    color: String,
}

pub struct Config {
    config_path: Option<String>,
    pub max_messages: usize,
    pub typewriter_delay: Duration,
    pub input_max_length: usize,
    pub max_history: usize,
    pub poll_rate: Duration,
    pub theme: Theme,
    pub prompt: Prompt,
    pub debug_info: Option<String>,
}

pub struct Theme {
    pub input_text: AppColor,
    pub input_bg: AppColor,
    pub cursor: AppColor,
    pub output_text: AppColor,
    pub output_bg: AppColor,
}

pub struct Prompt {
    pub text: String,
    pub color: AppColor,
}

impl Config {
    pub async fn load() -> Result<Self> {
        let mut last_error = None;

        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                match Self::from_file(&path).await {
                    Ok(config) => {
                        log::debug!("Konfiguration geladen: {}", path.display());
                        return Ok(config);
                    }
                    Err(e) => {
                        last_error = Some(e);
                        continue;
                    }
                }
            }
        }

        log::info!("Keine existierende Konfiguration, erstelle Standard");

        match crate::setup::setup_toml::ensure_config_exists().await {
            Ok(config_path) => match Self::from_file(&config_path).await {
                Ok(mut config) => {
                    // Nur 1x loggen!
                    let plain_msg =
                        format!("Neue Standard-Konfiguration in '{}'", config_path.display());
                    log::info!("{}", plain_msg);

                    // Nur zur internen Anzeige gespeichert, nicht nochmal geloggt
                    config.debug_info = Some(plain_msg);
                    Ok(config)
                }
                Err(e) => {
                    log::error!("Fehler beim Laden neuer Konfiguration: {:?}", e);
                    Err(e)
                }
            },
            Err(e) => {
                log::error!("Standard-Konfiguration fehlgeschlagen: {:?}", e);
                if let Some(last_e) = last_error {
                    log::debug!("Letzter Fehler: {:?}", last_e);
                }
                Err(e)
            }
        }
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(AppError::Io)?;

        let config_file: ConfigFile = toml::from_str(&content)
            .map_err(|e| AppError::Validation(format!("Ungültiges TOML-Format: {}", e)))?;

        Ok(Self {
            config_path: Some(path.as_ref().to_string_lossy().into_owned()),
            max_messages: config_file.general.max_messages,
            typewriter_delay: Duration::from_millis(config_file.general.typewriter_delay),
            input_max_length: config_file.general.input_max_length,
            max_history: config_file.general.max_history,
            poll_rate: Duration::from_millis(config_file.general.poll_rate),
            theme: Theme::from_config(&config_file.theme)?,
            prompt: Prompt::from_config(&config_file.prompt)?,
            debug_info: None,
        })
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            let config_file = ConfigFile {
                general: GeneralConfig {
                    max_messages: self.max_messages,
                    typewriter_delay: self.typewriter_delay.as_millis() as u64,
                    input_max_length: self.input_max_length,
                    max_history: self.max_history,
                    poll_rate: self.poll_rate.as_millis() as u64,
                },
                theme: ThemeConfig {
                    input_text: self.theme.input_text.to_string(),
                    input_bg: self.theme.input_bg.to_string(),
                    cursor: self.theme.cursor.to_string(),
                    output_text: self.theme.output_text.to_string(),
                    output_bg: self.theme.output_bg.to_string(),
                },
                prompt: PromptConfig {
                    text: self.prompt.text.clone(),
                    color: self.prompt.color.to_string(),
                },
            };

            let content = toml::to_string_pretty(&config_file)
                .map_err(|e| AppError::Validation(format!("Serialisierungsfehler: {}", e)))?;

            // Stelle sicher, dass das Verzeichnis existiert
            if let Some(parent) = PathBuf::from(path).parent() {
                if !parent.exists() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(AppError::Io)?;
                }
            }

            tokio::fs::write(path, content)
                .await
                .map_err(AppError::Io)?;
        }
        Ok(())
    }
}

impl Theme {
    fn from_config(config: &ThemeConfig) -> Result<Self> {
        Ok(Self {
            input_text: AppColor::from_string(&config.input_text)?,
            input_bg: AppColor::from_string(&config.input_bg)?,
            cursor: AppColor::from_string(&config.cursor)?,
            output_text: AppColor::from_string(&config.output_text)?,
            output_bg: AppColor::from_string(&config.output_bg)?,
        })
    }
}

impl Prompt {
    fn from_config(config: &PromptConfig) -> Result<Self> {
        Ok(Self {
            text: config.text.clone(),
            color: AppColor::from_string(&config.color)?,
        })
    }
}

// Default-Implementierungen bleiben unverändert
impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: None,
            max_messages: DEFAULT_BUFFER_SIZE,
            typewriter_delay: Duration::from_millis(50),
            input_max_length: DEFAULT_BUFFER_SIZE,
            max_history: 30,
            poll_rate: Duration::from_millis(DEFAULT_POLL_RATE),
            theme: Theme::default(),
            prompt: Prompt::default(),
            debug_info: None,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            input_text: AppColor::new(Color::Black),
            input_bg: AppColor::new(Color::Black),
            cursor: AppColor::new(Color::Black),
            output_text: AppColor::new(Color::White),
            output_bg: AppColor::new(Color::White),
        }
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            text: "/// ".to_string(),
            color: AppColor::new(Color::Black),
        }
    }
}
