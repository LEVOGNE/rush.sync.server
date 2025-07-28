// =====================================================
// FILE: src/core/config.rs - VOLLSTÄNDIG mit BOUNDS CHECKING
// =====================================================

use crate::core::constants::{DEFAULT_BUFFER_SIZE, DEFAULT_POLL_RATE};
use crate::core::prelude::*;
use crate::ui::color::AppColor;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ✅ SICHERE BOUNDS für Performance
const MIN_POLL_RATE: u64 = 16; // 60 FPS maximum
const MAX_POLL_RATE: u64 = 1000; // 1 FPS minimum
const MAX_TYPEWRITER_DELAY: u64 = 2000; // Maximum 2 Sekunden

// ✅ ALLE STRUCT DEFINITIONEN (vorher fehlten die!)
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
    max_history: usize,
    poll_rate: u64,
    log_level: String,
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

// ✅ HAUPTSTRUCTS (müssen public sein!)
pub struct Config {
    config_path: Option<String>,
    pub max_messages: usize,
    pub typewriter_delay: Duration,
    pub input_max_length: usize,
    pub max_history: usize,
    pub poll_rate: Duration,
    pub log_level: String,
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
        // ✅ 1. PRÜFE ob Config bereits existiert
        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                match Self::from_file(&path).await {
                    Ok(config) => {
                        // ✅ PERFORMANCE WARNING nur bei problematischen Werten
                        if config.poll_rate.as_millis() < 16 && show_messages {
                            log::warn!(
                                "⚡ PERFORMANCE: poll_rate sehr niedrig! {}",
                                config.get_performance_info()
                            );
                        }

                        // Sprache setzen (ohne Log-Spam)
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

                        // ✅ VERSION nur einmal beim echten Start
                        if show_messages {
                            crate::output::logging::AppLogger::log_plain(
                                crate::i18n::get_command_translation(
                                    "system.startup.version",
                                    &[crate::core::constants::VERSION],
                                ),
                            );
                        }

                        return Ok(config);
                    }
                    Err(_e) => {
                        continue;
                    }
                }
            }
        }

        // ✅ 2. KEINE CONFIGS GEFUNDEN - Neue erstellen
        if show_messages {
            log::info!("{}", get_translation("system.config.no_existing", &[]));
        }

        match crate::setup::setup_toml::ensure_config_exists().await {
            Ok(config_path) => {
                match Self::from_file(&config_path).await {
                    Ok(mut config) => {
                        // ✅ NUR BEI FIRST-RUN zeigen
                        if show_messages {
                            let plain_msg = get_translation(
                                "system.config.new_default",
                                &[&config_path.display().to_string()],
                            );
                            log::info!("{}", plain_msg);
                            config.debug_info = Some(plain_msg);

                            crate::output::logging::AppLogger::log_plain(
                                crate::i18n::get_command_translation(
                                    "system.startup.version",
                                    &[crate::core::constants::VERSION],
                                ),
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

        // ✅ BOUNDS CHECKING mit Warnungen
        let original_poll_rate = config_file.general.poll_rate;
        let original_typewriter_delay = config_file.general.typewriter_delay;

        let poll_rate = Self::validate_poll_rate(original_poll_rate);
        let typewriter_delay = Self::validate_typewriter_delay(original_typewriter_delay);

        let config = Self {
            config_path: Some(path.as_ref().to_string_lossy().into_owned()),
            max_messages: config_file.general.max_messages,
            typewriter_delay: Duration::from_millis(typewriter_delay),
            input_max_length: config_file.general.input_max_length,
            max_history: config_file.general.max_history,
            poll_rate: Duration::from_millis(poll_rate),
            log_level: config_file.general.log_level,
            theme: Theme::from_config(&config_file.theme)?,
            prompt: Prompt::from_config(&config_file.prompt)?,
            language: config_file.language.current,
            debug_info: None,
        };

        // ✅ KORRIGIERTE WERTE ZURÜCKSCHREIBEN (falls geändert)
        let values_changed =
            original_poll_rate != poll_rate || original_typewriter_delay != typewriter_delay;

        if values_changed {
            log::warn!("🔧 Ungültige Config-Werte korrigiert und gespeichert:");
            if original_poll_rate != poll_rate {
                log::warn!("   poll_rate: {}ms → {}ms", original_poll_rate, poll_rate);
            }
            if original_typewriter_delay != typewriter_delay {
                log::warn!(
                    "   typewriter_delay: {}ms → {}ms",
                    original_typewriter_delay,
                    typewriter_delay
                );
            }

            // ✅ SOFORT ZURÜCKSCHREIBEN damit beim nächsten Start korrekte Werte geladen werden
            if let Err(e) = config.save().await {
                log::warn!("Konnte korrigierte Config nicht speichern: {}", e);
            } else {
                log::info!("✅ Korrigierte Werte in Config-Datei gespeichert");
            }
        }

        Ok(config)
    }

    // ✅ POLL_RATE Validierung
    fn validate_poll_rate(value: u64) -> u64 {
        match value {
            0 => {
                log::warn!(
                    "poll_rate = 0 nicht erlaubt, verwende Minimum: {}ms",
                    MIN_POLL_RATE
                );
                MIN_POLL_RATE
            }
            v if v < MIN_POLL_RATE => {
                log::warn!(
                    "poll_rate = {}ms zu schnell (Performance!), verwende Minimum: {}ms",
                    v,
                    MIN_POLL_RATE
                );
                MIN_POLL_RATE
            }
            v if v > MAX_POLL_RATE => {
                log::warn!(
                    "poll_rate = {}ms zu langsam, verwende Maximum: {}ms",
                    v,
                    MAX_POLL_RATE
                );
                MAX_POLL_RATE
            }
            v => {
                if v < 33 {
                    log::trace!("poll_rate = {}ms (sehr schnell, aber OK)", v);
                }
                v
            }
        }
    }

    // ✅ TYPEWRITER_DELAY Validierung (0 = deaktiviert bleibt 0!)
    fn validate_typewriter_delay(value: u64) -> u64 {
        match value {
            0 => {
                log::info!("typewriter_delay = 0 → Typewriter-Effekt deaktiviert");
                0 // ✅ BLEIBT 0 für echte Deaktivierung!
            }
            v if v > MAX_TYPEWRITER_DELAY => {
                log::warn!(
                    "typewriter_delay = {}ms zu langsam, verwende Maximum: {}ms",
                    v,
                    MAX_TYPEWRITER_DELAY
                );
                MAX_TYPEWRITER_DELAY
            }
            v => v,
        }
    }

    // ✅ PERFORMANCE INFO für Debug
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

    pub async fn save(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            let config_file = ConfigFile {
                general: GeneralConfig {
                    max_messages: self.max_messages,
                    typewriter_delay: self.typewriter_delay.as_millis() as u64,
                    input_max_length: self.input_max_length,
                    max_history: self.max_history,
                    poll_rate: self.poll_rate.as_millis() as u64,
                    log_level: self.log_level.clone(),
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

crate::impl_default!(
    Config,
    Self {
        config_path: None,
        max_messages: DEFAULT_BUFFER_SIZE,
        typewriter_delay: Duration::from_millis(50), // ✅ Sicherer Default
        input_max_length: DEFAULT_BUFFER_SIZE,
        max_history: 30,
        poll_rate: Duration::from_millis(DEFAULT_POLL_RATE), // ✅ 16ms = 60fps
        log_level: "info".to_string(),
        theme: Theme::default(),
        prompt: Prompt::default(),
        language: crate::i18n::DEFAULT_LANGUAGE.to_string(),
        debug_info: None,
    }
);

crate::impl_default!(
    Theme,
    Self {
        input_text: AppColor::new(Color::Black),
        input_bg: AppColor::new(Color::Black),
        cursor: AppColor::new(Color::Black),
        output_text: AppColor::new(Color::White),
        output_bg: AppColor::new(Color::White),
    }
);

crate::impl_default!(
    Prompt,
    Self {
        text: "/// ".to_string(),
        color: AppColor::new(Color::Black),
    }
);

// ✅ DEBUG: Performance-Warnung zur Compile-Zeit
#[cfg(debug_assertions)]
impl Config {
    pub fn debug_performance_warning(&self) {
        if self.poll_rate.as_millis() < 16 {
            log::warn!(
                "🔥 PERFORMANCE WARNING: poll_rate = {}ms verursacht hohe CPU-Last!",
                self.poll_rate.as_millis()
            );
            log::warn!("💡 EMPFEHLUNG: Setze poll_rate auf 16-33ms für bessere Performance");
        }

        if self.typewriter_delay.as_millis() < 10 {
            log::warn!(
                "⚡ PERFORMANCE INFO: typewriter_delay = {}ms (sehr schnell)",
                self.typewriter_delay.as_millis()
            );
        }

        log::info!("📊 AKTUELLE WERTE: {}", self.get_performance_info());
    }
}
