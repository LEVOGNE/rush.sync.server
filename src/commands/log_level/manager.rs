use log::LevelFilter;
use std::sync::Mutex;

/// Globaler Log-Level Manager für Runtime-Änderungen + Persistenz
pub struct LogLevelManager;

// Globaler STATE für aktuelles Log-Level
static CURRENT_LOG_LEVEL: Mutex<LevelFilter> = Mutex::new(LevelFilter::Info);

impl LogLevelManager {
    /// Zeigt aktuelles Log-Level und verfügbare Optionen
    pub fn show_status() -> String {
        let current = Self::get_current_level();
        let current_name = Self::level_to_name(current);
        let current_number = Self::level_to_number(current);

        format!(
            "{}: {} ({})\n{}",
            crate::i18n::get_command_translation("system.commands.log_level.current", &[]),
            current_name,
            current_number,
            Self::show_help()
        )
    }

    /// Setzt neues Log-Level UND speichert in Config
    pub fn set_level_persistent(level_input: &str) -> Result<String, String> {
        let level_filter = match level_input {
            "1" => LevelFilter::Error,
            "2" => LevelFilter::Warn,
            "3" => LevelFilter::Info,
            "4" => LevelFilter::Debug,
            "5" => LevelFilter::Trace,
            "error" | "ERROR" => LevelFilter::Error,
            "warn" | "WARN" | "warning" => LevelFilter::Warn,
            "info" | "INFO" => LevelFilter::Info,
            "debug" | "DEBUG" => LevelFilter::Debug,
            "trace" | "TRACE" => LevelFilter::Trace,
            _ => {
                return Err(crate::i18n::get_command_translation(
                    "system.commands.log_level.invalid",
                    &[level_input],
                ));
            }
        };

        // ✅ 1. SETZE RUNTIME LOG-LEVEL sofort
        Self::set_level_runtime(level_filter);

        // ✅ 2. SPAWN BACKGROUND-TASK für Config-Save
        tokio::spawn(async move {
            if let Err(e) = Self::save_to_config(level_filter).await {
                log::warn!("Failed to save log level to config: {}", e);
            } else {
                log::debug!("Log level saved to config successfully");
            }
        });

        let level_name = Self::level_to_name(level_filter);
        let level_number = Self::level_to_number(level_filter);

        Ok(crate::i18n::get_command_translation(
            "system.commands.log_level.changed_persistent",
            &[&level_name, &level_number],
        ))
    }

    /// Setzt nur Runtime Log-Level (ohne Config-Save)
    pub fn set_level_runtime(level_filter: LevelFilter) {
        if let Ok(mut current) = CURRENT_LOG_LEVEL.lock() {
            *current = level_filter;
        }
        log::set_max_level(level_filter);
    }

    /// Lädt Log-Level aus Config beim Startup
    pub async fn load_from_config() -> LevelFilter {
        match crate::core::config::Config::load_with_messages(false).await {
            Ok(config) => match Self::string_to_level_filter(&config.log_level) {
                Ok(level) => {
                    log::debug!("Log level loaded from config: {}", config.log_level);
                    level
                }
                Err(_) => {
                    log::warn!(
                        "Invalid log level in config: '{}', using INFO",
                        config.log_level
                    );
                    LevelFilter::Info
                }
            },
            Err(_) => {
                log::debug!("No config found, using default log level: INFO");
                LevelFilter::Info
            }
        }
    }

    /// Speichert Log-Level in Config-Datei
    async fn save_to_config(level_filter: LevelFilter) -> Result<(), String> {
        match crate::core::config::Config::load_with_messages(false).await {
            Ok(mut config) => {
                config.log_level = Self::level_filter_to_string(level_filter);
                config
                    .save()
                    .await
                    .map_err(|e| format!("Config save error: {}", e))
            }
            Err(e) => Err(format!("Config load error: {}", e)),
        }
    }

    /// Aktuelles Log-Level abrufen
    pub fn get_current_level() -> LevelFilter {
        if let Ok(current) = CURRENT_LOG_LEVEL.lock() {
            *current
        } else {
            log::max_level()
        }
    }

    /// Initialisiert Log-Level (für startup)
    pub fn init_with_level(level: LevelFilter) {
        if let Ok(mut current) = CURRENT_LOG_LEVEL.lock() {
            *current = level;
        }
        log::set_max_level(level);
    }

    /// Hilfe-Text
    pub fn show_help() -> String {
        crate::i18n::get_command_translation("system.commands.log_level.help", &[])
    }

    // ✅ HELPER: String zu LevelFilter
    fn string_to_level_filter(s: &str) -> Result<LevelFilter, ()> {
        match s.to_lowercase().as_str() {
            "error" | "1" => Ok(LevelFilter::Error),
            "warn" | "warning" | "2" => Ok(LevelFilter::Warn),
            "info" | "3" => Ok(LevelFilter::Info),
            "debug" | "4" => Ok(LevelFilter::Debug),
            "trace" | "5" => Ok(LevelFilter::Trace),
            "off" | "0" => Ok(LevelFilter::Off),
            _ => Err(()),
        }
    }

    // ✅ HELPER: LevelFilter zu String
    fn level_filter_to_string(level: LevelFilter) -> String {
        match level {
            LevelFilter::Error => "error".to_string(),
            LevelFilter::Warn => "warn".to_string(),
            LevelFilter::Info => "info".to_string(),
            LevelFilter::Debug => "debug".to_string(),
            LevelFilter::Trace => "trace".to_string(),
            LevelFilter::Off => "off".to_string(),
        }
    }

    // ✅ HELPER: Level zu Name (Display)
    fn level_to_name(level: LevelFilter) -> String {
        match level {
            LevelFilter::Error => "ERROR".to_string(),
            LevelFilter::Warn => "WARN".to_string(),
            LevelFilter::Info => "INFO".to_string(),
            LevelFilter::Debug => "DEBUG".to_string(),
            LevelFilter::Trace => "TRACE".to_string(),
            LevelFilter::Off => "OFF".to_string(),
        }
    }

    // ✅ HELPER: Level zu Nummer
    fn level_to_number(level: LevelFilter) -> String {
        match level {
            LevelFilter::Error => "1".to_string(),
            LevelFilter::Warn => "2".to_string(),
            LevelFilter::Info => "3".to_string(),
            LevelFilter::Debug => "4".to_string(),
            LevelFilter::Trace => "5".to_string(),
            LevelFilter::Off => "0".to_string(),
        }
    }
}
