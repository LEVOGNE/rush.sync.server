// =====================================================
// FILE: core/config.rs - BEREINIGT nach History-Refactoring
// =====================================================

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
    language: LanguageConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneralConfig {
    max_messages: usize,
    typewriter_delay: u64,
    input_max_length: usize,
    max_history: usize, // ✅ BLEIBT für Rückwärtskompatibilität
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

#[derive(Debug, Serialize, Deserialize)]
struct LanguageConfig {
    current: String,
}

pub struct Config {
    config_path: Option<String>,
    pub max_messages: usize,
    pub typewriter_delay: Duration,
    pub input_max_length: usize,
    pub max_history: usize, // ✅ BLEIBT - wird von HistoryConfig verwendet
    pub poll_rate: Duration,
    pub theme: Theme,
    pub prompt: Prompt,
    pub language: String,
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
        Self::load_with_messages(true).await
    }

    pub async fn load_with_messages(show_messages: bool) -> Result<Self> {
        let mut last_error = None;

        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                match Self::from_file(&path).await {
                    Ok(config) => {
                        // ✅ SPRACHE DELEGIERT an LanguageConfig
                        if let Err(e) = crate::commands::lang::config::LanguageConfig::load_and_apply_from_config(&config).await {
                            if show_messages {
                                log::warn!(
                                    "{}",
                                    get_translation(
                                        "system.config.language_set_failed",
                                        &[&e.to_string()]
                                    )
                                );
                            }
                        }

                        if show_messages {
                            log::debug!(
                                "{}",
                                get_translation(
                                    "system.config.loaded",
                                    &[&path.display().to_string()]
                                )
                            );

                            log::info!(
                                "{}",
                                crate::i18n::get_command_translation(
                                    "system.startup.version",
                                    &[crate::core::constants::VERSION]
                                )
                            );
                        }

                        return Ok(config);
                    }
                    Err(e) => {
                        last_error = Some(e);
                        continue;
                    }
                }
            }
        }

        if show_messages {
            log::info!("{}", get_translation("system.config.no_existing", &[]));
        }

        match crate::setup::setup_toml::ensure_config_exists().await {
            Ok(config_path) => {
                match Self::from_file(&config_path).await {
                    Ok(mut config) => {
                        if show_messages {
                            let plain_msg = get_translation(
                                "system.config.new_default",
                                &[&config_path.display().to_string()],
                            );
                            log::info!("{}", plain_msg);
                            config.debug_info = Some(plain_msg);

                            log::info!(
                                "{}",
                                crate::i18n::get_command_translation(
                                    "system.startup.version",
                                    &[crate::core::constants::VERSION]
                                )
                            );
                        }

                        let _ = crate::commands::lang::config::LanguageConfig::load_and_apply_from_config(&config).await;

                        Ok(config)
                    }
                    Err(e) => {
                        if show_messages {
                            log::error!(
                                "{}",
                                get_translation("system.config.load_error", &[&format!("{:?}", e)])
                            );
                        }
                        Err(e)
                    }
                }
            }
            Err(e) => {
                if show_messages {
                    log::error!(
                        "{}",
                        get_translation("system.config.setup_failed", &[&format!("{:?}", e)])
                    );
                    if let Some(last_e) = last_error {
                        log::debug!(
                            "{}",
                            get_translation(
                                "system.config.last_error",
                                &[&format!("{:?}", last_e)]
                            )
                        );
                    }
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
            max_history: config_file.general.max_history, // ✅ BLEIBT für HistoryConfig
            poll_rate: Duration::from_millis(config_file.general.poll_rate),
            theme: Theme::from_config(&config_file.theme)?,
            prompt: Prompt::from_config(&config_file.prompt)?,
            language: config_file.language.current,
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
                    max_history: self.max_history, // ✅ BLEIBT
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
                language: LanguageConfig {
                    current: self.language.clone(),
                },
            };

            let content = toml::to_string_pretty(&config_file)
                .map_err(|e| AppError::Validation(format!("Serialisierungsfehler: {}", e)))?;

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

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: None,
            max_messages: DEFAULT_BUFFER_SIZE,
            typewriter_delay: Duration::from_millis(50),
            input_max_length: DEFAULT_BUFFER_SIZE,
            max_history: 30, // ✅ BLEIBT - wird von HistoryConfig verwendet
            poll_rate: Duration::from_millis(DEFAULT_POLL_RATE),
            theme: Theme::default(),
            prompt: Prompt::default(),
            language: crate::i18n::DEFAULT_LANGUAGE.to_string(),
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
