// ## FILE: src/core/config.rs - KOMPRIMIERTE VERSION
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
    #[serde(default = "default_theme")]
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
    #[serde(default = "default_prefix")]
    input_cursor_prefix: String,
    #[serde(default = "default_input_color")]
    input_cursor_color: String,
    #[serde(default = "default_cursor")]
    input_cursor: String,
    #[serde(default = "default_cursor")]
    output_cursor: String,
    #[serde(default = "default_output_color")]
    output_cursor_color: String,
}

// ✅ KOMPRIMIERTE DEFAULTS
fn default_theme() -> String {
    "dark".into()
}
fn default_prefix() -> String {
    "/// ".into()
}
fn default_input_color() -> String {
    "LightBlue".into()
}
fn default_output_color() -> String {
    "White".into()
}
fn default_cursor() -> String {
    "PIPE".into()
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
            input_cursor_prefix: "/// ".into(),
            input_cursor_color: AppColor::new(Color::LightBlue),
            input_cursor: "PIPE".into(),
            output_cursor: "PIPE".into(),
            output_cursor_color: AppColor::new(Color::White),
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        Self::load_with_messages(true).await
    }

    pub async fn load_with_messages(show_messages: bool) -> Result<Self> {
        // Try existing configs
        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                if let Ok(config) = Self::from_file(&path).await {
                    if show_messages {
                        Self::log_startup(&config);
                    }
                    Self::apply_language(&config).await;
                    return Ok(config);
                }
            }
        }

        // Create new config
        let path = crate::setup::setup_toml::ensure_config_exists().await?;
        let mut config = Self::from_file(&path).await?;

        if show_messages {
            config.debug_info = Some(format!("New config: {}", path.display()));
            Self::log_startup(&config);
        }

        Self::apply_language(&config).await;
        Ok(config)
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(AppError::Io)?;
        let file: ConfigFile =
            toml::from_str(&content).map_err(|e| AppError::Validation(format!("TOML: {}", e)))?;

        let poll_rate = Self::clamp(file.general.poll_rate, 16, 1000, 16);
        let typewriter = Self::clamp(file.general.typewriter_delay, 0, 2000, 50);
        let theme = Self::load_theme(&file).unwrap_or_default();

        let config = Self {
            config_path: Some(path.as_ref().to_string_lossy().into_owned()),
            max_messages: file.general.max_messages,
            typewriter_delay: Duration::from_millis(typewriter),
            input_max_length: file.general.input_max_length,
            max_history: file.general.max_history,
            poll_rate: Duration::from_millis(poll_rate),
            log_level: file.general.log_level,
            theme,
            current_theme_name: file.general.current_theme,
            language: file.language.current,
            debug_info: None,
        };

        // Auto-save corrected values
        if poll_rate != file.general.poll_rate || typewriter != file.general.typewriter_delay {
            let _ = config.save().await;
        }

        Ok(config)
    }

    fn clamp(value: u64, min: u64, max: u64, default: u64) -> u64 {
        if value < min || value > max {
            default
        } else {
            value
        }
    }

    fn load_theme(file: &ConfigFile) -> Option<Theme> {
        let themes = file.theme.as_ref()?;
        let def = themes.get(&file.general.current_theme)?;
        Theme::from_config(def).ok()
    }

    // ✅ KOMPRIMIERTES SAVE
    pub async fn save(&self) -> Result<()> {
        let Some(path) = &self.config_path else {
            return Ok(());
        };

        let themes = Self::load_existing_themes().await.unwrap_or_default();
        let file = ConfigFile {
            general: GeneralConfig {
                max_messages: self.max_messages,
                typewriter_delay: self.typewriter_delay.as_millis() as u64,
                input_max_length: self.input_max_length,
                max_history: self.max_history,
                poll_rate: self.poll_rate.as_millis() as u64,
                log_level: self.log_level.clone(),
                current_theme: self.current_theme_name.clone(),
            },
            theme: if themes.is_empty() {
                None
            } else {
                Some(themes)
            },
            language: LanguageConfig {
                current: self.language.clone(),
            },
        };

        let content = toml::to_string_pretty(&file)
            .map_err(|e| AppError::Validation(format!("TOML: {}", e)))?;

        // Ensure dir exists
        if let Some(parent) = std::path::PathBuf::from(path).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::Io)?;
        }

        tokio::fs::write(path, content).await.map_err(AppError::Io)
    }

    // ✅ KOMPRIMIERTES THEME SWITCHING
    pub async fn change_theme(&mut self, name: &str) -> Result<()> {
        let themes = Self::load_existing_themes().await?;
        let def = themes
            .get(name)
            .ok_or_else(|| AppError::Validation(format!("Theme '{}' not found", name)))?;

        self.theme = Theme::from_config(def)?;
        self.current_theme_name = name.into();
        self.save().await
    }

    async fn load_existing_themes() -> Result<HashMap<String, ThemeDefinitionConfig>> {
        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AppError::Io)?;
                let file: ConfigFile = toml::from_str(&content)
                    .map_err(|e| AppError::Validation(format!("TOML: {}", e)))?;

                if let Some(themes) = file.theme {
                    return Ok(themes);
                }
            }
        }
        Ok(HashMap::new())
    }

    // ✅ PERFORMANCE INFO (komprimiert)
    pub fn get_performance_info(&self) -> String {
        let fps = 1000.0 / self.poll_rate.as_millis() as f64;
        let typewriter = if self.typewriter_delay.as_millis() > 0 {
            1000.0 / self.typewriter_delay.as_millis() as f64
        } else {
            f64::INFINITY
        };
        format!(
            "Performance: {:.1} FPS, Typewriter: {:.1} chars/sec",
            fps, typewriter
        )
    }

    // ✅ HELPER METHODS (komprimiert)
    async fn apply_language(config: &Config) {
        let _ = crate::commands::lang::LanguageService::new()
            .load_and_apply_from_config(config)
            .await;
    }

    fn log_startup(config: &Config) {
        if config.poll_rate.as_millis() < 16 {
            log::warn!("⚡ PERFORMANCE: poll_rate sehr niedrig!");
        }
        log::info!("Rush Sync Server v{}", crate::core::constants::VERSION);
    }
}

impl Theme {
    fn from_config(def: &ThemeDefinitionConfig) -> Result<Self> {
        Ok(Self {
            input_text: AppColor::from_string(&def.input_text)?,
            input_bg: AppColor::from_string(&def.input_bg)?,
            output_text: AppColor::from_string(&def.output_text)?,
            output_bg: AppColor::from_string(&def.output_bg)?,
            input_cursor_prefix: def.input_cursor_prefix.clone(),
            input_cursor_color: AppColor::from_string(&def.input_cursor_color)?,
            input_cursor: def.input_cursor.clone(),
            output_cursor: def.output_cursor.clone(),
            output_cursor_color: AppColor::from_string(&def.output_cursor_color)?,
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
            log_level: "info".into(),
            theme: Theme::default(),
            current_theme_name: "dark".into(),
            language: crate::i18n::DEFAULT_LANGUAGE.into(),
            debug_info: None,
        }
    }
}
