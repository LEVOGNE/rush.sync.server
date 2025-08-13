// =====================================================
// FILE: src/core/config.rs - BEREINIGTE THEME-KONVERTIERUNG
// =====================================================

use crate::core::constants::{DEFAULT_BUFFER_SIZE, DEFAULT_POLL_RATE};
use crate::core::prelude::*;
use crate::ui::color::AppColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    general: GeneralConfig,
    #[serde(default)]
    theme: Option<HashMap<String, ThemeDefinitionConfig>>,
    language: LanguageConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneralConfig {
    max_messages: usize,
    typewriter_delay: u64,
    input_max_length: usize,
    max_history: usize,
    poll_rate: u64,
    log_level: String,
    #[serde(default = "default_theme_name")]
    current_theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LanguageConfig {
    current: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ThemeDefinitionConfig {
    input_text: String,
    input_bg: String,
    output_text: String,
    output_bg: String,

    // ✅ DIREKTE FELD-ZUORDNUNG (Feld-Namen stimmen mit Config-Struct überein)
    #[serde(default = "default_input_cursor_prefix")]
    input_cursor_prefix: String,

    #[serde(default = "default_input_cursor_color")]
    input_cursor_color: String,

    #[serde(default = "default_input_cursor")]
    input_cursor: String,

    #[serde(default = "default_output_cursor")]
    output_cursor: String,

    #[serde(default = "default_output_cursor_color")]
    output_cursor_color: String,
}

fn default_theme_name() -> String {
    "dark".to_string()
}

// ✅ KONSISTENTE CURSOR-DEFAULTS
fn default_input_cursor_prefix() -> String {
    "/// ".to_string()
}
fn default_input_cursor_color() -> String {
    "LightBlue".to_string()
}
fn default_input_cursor() -> String {
    "PIPE".to_string()
}
fn default_output_cursor() -> String {
    "PIPE".to_string()
}
fn default_output_cursor_color() -> String {
    "White".to_string()
}

#[derive(Clone)]
pub struct Config {
    config_path: Option<String>,
    pub max_messages: usize,
    pub typewriter_delay: Duration,
    pub input_max_length: usize,
    pub max_history: usize,
    pub poll_rate: Duration,
    pub log_level: String,
    pub theme: Theme,
    pub current_theme_name: String,
    pub language: String,
    pub debug_info: Option<String>,
}

#[derive(Clone)]
pub struct Theme {
    pub input_text: AppColor,
    pub input_bg: AppColor,
    pub output_text: AppColor,
    pub output_bg: AppColor,

    // ✅ EXAKTE FELD-ÜBEREINSTIMMUNG mit ThemeDefinition
    pub input_cursor_prefix: String,
    pub input_cursor_color: AppColor,
    pub input_cursor: String,
    pub output_cursor: String,
    pub output_cursor_color: AppColor,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            input_text: AppColor::new(Color::White),
            input_bg: AppColor::new(Color::Black),
            output_text: AppColor::new(Color::White),
            output_bg: AppColor::new(Color::Black),

            // ✅ KONSISTENTE DEFAULTS
            input_cursor_prefix: "/// ".to_string(),
            input_cursor_color: AppColor::new(Color::LightBlue),
            input_cursor: "PIPE".to_string(),
            output_cursor: "PIPE".to_string(),
            output_cursor_color: AppColor::new(Color::White),
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        Self::load_with_messages(true).await
    }

    pub async fn load_with_messages(show_messages: bool) -> Result<Self> {
        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                if let Ok(config) = Self::from_file(&path).await {
                    if show_messages && config.poll_rate.as_millis() < 16 {
                        log::warn!("⚡ PERFORMANCE: poll_rate sehr niedrig!");
                    }

                    let _ = crate::commands::lang::LanguageService::new()
                        .load_and_apply_from_config(&config)
                        .await;

                    if show_messages {
                        log::info!("Rush Sync Server v{}", crate::core::constants::VERSION);
                    }
                    return Ok(config);
                }
            }
        }

        if show_messages {
            log::info!("Keine Config gefunden, erstelle neue");
        }

        let config_path = crate::setup::setup_toml::ensure_config_exists().await?;
        let mut config = Self::from_file(&config_path).await?;

        if show_messages {
            config.debug_info = Some(format!("Neue Config erstellt: {}", config_path.display()));
            log::info!("Rush Sync Server v{}", crate::core::constants::VERSION);
        }

        let _ = crate::commands::lang::LanguageService::new()
            .load_and_apply_from_config(&config)
            .await;

        Ok(config)
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(AppError::Io)?;
        let config_file: ConfigFile = toml::from_str(&content)
            .map_err(|e| AppError::Validation(format!("TOML Error: {}", e)))?;

        let poll_rate = Self::validate_range(config_file.general.poll_rate, 16, 1000, 16);
        let typewriter_delay =
            Self::validate_range(config_file.general.typewriter_delay, 0, 2000, 50);

        let theme = Self::load_theme_from_config(&config_file)?;

        let config = Self {
            config_path: Some(path.as_ref().to_string_lossy().into_owned()),
            max_messages: config_file.general.max_messages,
            typewriter_delay: Duration::from_millis(typewriter_delay),
            input_max_length: config_file.general.input_max_length,
            max_history: config_file.general.max_history,
            poll_rate: Duration::from_millis(poll_rate),
            log_level: config_file.general.log_level,
            theme,
            current_theme_name: config_file.general.current_theme,
            language: config_file.language.current,
            debug_info: None,
        };

        if poll_rate != config_file.general.poll_rate
            || typewriter_delay != config_file.general.typewriter_delay
        {
            log::warn!("Config-Werte korrigiert und gespeichert");
            let _ = config.save().await;
        }

        Ok(config)
    }

    fn validate_range(value: u64, min: u64, max: u64, default: u64) -> u64 {
        if value < min || value > max {
            log::warn!(
                "Wert {} außerhalb Bereich {}-{}, verwende {}",
                value,
                min,
                max,
                default
            );
            default
        } else {
            value
        }
    }

    fn load_theme_from_config(config_file: &ConfigFile) -> Result<Theme> {
        let current_theme_name = &config_file.general.current_theme;

        if let Some(ref themes) = config_file.theme {
            if let Some(theme_def) = themes.get(current_theme_name) {
                return Theme::from_config(theme_def);
            }
        }

        log::warn!(
            "Theme '{}' nicht gefunden, verwende Standard",
            current_theme_name
        );
        Ok(Theme::default())
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            let existing_themes = Self::load_themes_from_config().await.unwrap_or_default();

            let config_file = ConfigFile {
                general: GeneralConfig {
                    max_messages: self.max_messages,
                    typewriter_delay: self.typewriter_delay.as_millis() as u64,
                    input_max_length: self.input_max_length,
                    max_history: self.max_history,
                    poll_rate: self.poll_rate.as_millis() as u64,
                    log_level: self.log_level.clone(),
                    current_theme: self.current_theme_name.clone(),
                },
                theme: if existing_themes.is_empty() {
                    None
                } else {
                    Some(existing_themes)
                },
                language: LanguageConfig {
                    current: self.language.clone(),
                },
            };

            let content = toml::to_string_pretty(&config_file)
                .map_err(|e| AppError::Validation(format!("TOML Error: {}", e)))?;

            if let Some(parent) = std::path::PathBuf::from(path).parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(AppError::Io)?;
            }

            tokio::fs::write(path, content)
                .await
                .map_err(AppError::Io)?;
        }
        Ok(())
    }

    pub async fn change_theme(&mut self, theme_name: &str) -> Result<()> {
        let available_themes = Self::load_themes_from_config().await?;

        if let Some(theme_def) = available_themes.get(theme_name) {
            self.theme = Theme::from_config(theme_def)?;
            self.current_theme_name = theme_name.to_string();
            self.save().await?;
            log::info!("Theme gewechselt zu: {}", theme_name);
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Theme '{}' nicht gefunden",
                theme_name
            )))
        }
    }

    async fn load_themes_from_config() -> Result<HashMap<String, ThemeDefinitionConfig>> {
        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AppError::Io)?;
                let config_file: ConfigFile = toml::from_str(&content)
                    .map_err(|e| AppError::Validation(format!("TOML Error: {}", e)))?;

                if let Some(themes) = config_file.theme {
                    return Ok(themes);
                }
            }
        }
        Ok(HashMap::new())
    }

    pub fn get_performance_info(&self) -> String {
        let fps = 1000.0 / self.poll_rate.as_millis() as f64;
        let typewriter_chars_per_sec = if self.typewriter_delay.as_millis() > 0 {
            1000.0 / self.typewriter_delay.as_millis() as f64
        } else {
            f64::INFINITY
        };

        format!(
            "Performance: {:.1} FPS, Typewriter: {:.1} chars/sec",
            fps, typewriter_chars_per_sec
        )
    }
}

impl Theme {
    // ✅ KOMPLETT BEREINIGTE Theme-Konvertierung OHNE Legacy-Fallbacks
    fn from_config(theme_def: &ThemeDefinitionConfig) -> Result<Self> {
        log::debug!(
            "🔧 Theme::from_config - DIREKTE KONVERTIERUNG:\n  \
            input_cursor_prefix: '{}'\n  \
            input_cursor_color: '{}'\n  \
            input_cursor: '{}'\n  \
            output_cursor: '{}'\n  \
            output_cursor_color: '{}'",
            theme_def.input_cursor_prefix,
            theme_def.input_cursor_color,
            theme_def.input_cursor,
            theme_def.output_cursor,
            theme_def.output_cursor_color
        );

        Ok(Self {
            input_text: AppColor::from_string(&theme_def.input_text)?,
            input_bg: AppColor::from_string(&theme_def.input_bg)?,
            output_text: AppColor::from_string(&theme_def.output_text)?,
            output_bg: AppColor::from_string(&theme_def.output_bg)?,

            // ✅ DIREKTE 1:1 ZUORDNUNG (keine Legacy-Behandlung!)
            input_cursor_prefix: theme_def.input_cursor_prefix.clone(),
            input_cursor_color: AppColor::from_string(&theme_def.input_cursor_color)?,
            input_cursor: theme_def.input_cursor.clone(),
            output_cursor: theme_def.output_cursor.clone(),
            output_cursor_color: AppColor::from_string(&theme_def.output_cursor_color)?,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: None,
            max_messages: DEFAULT_BUFFER_SIZE,
            typewriter_delay: Duration::from_millis(50),
            input_max_length: DEFAULT_BUFFER_SIZE,
            max_history: 30,
            poll_rate: Duration::from_millis(DEFAULT_POLL_RATE),
            log_level: "info".to_string(),
            theme: Theme::default(),
            current_theme_name: "dark".to_string(),
            language: crate::i18n::DEFAULT_LANGUAGE.to_string(),
            debug_info: None,
        }
    }
}
