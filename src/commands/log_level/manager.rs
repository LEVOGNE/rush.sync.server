use crate::core::prelude::*;
use log::LevelFilter;
use std::sync::Mutex;

pub struct LogLevelManager;

static CURRENT_LOG_LEVEL: Mutex<LevelFilter> = Mutex::new(LevelFilter::Info);

impl LogLevelManager {
    // ✅ STATUS DISPLAY mit i18n
    pub fn show_status() -> String {
        let current = Self::get_current_level();
        let current_name = Self::level_to_name(current);
        let current_number = Self::level_to_number(current);

        format!(
            "{}\n{}",
            get_command_translation(
                "system.commands.log_level.current_status",
                &[&current_name, &current_number]
            ),
            Self::show_help_i18n()
        )
    }

    // ✅ HELP TEXT mit i18n
    pub fn show_help_i18n() -> String {
        get_command_translation("system.commands.log_level.help_text", &[])
    }

    // ✅ Legacy method (for compatibility)
    pub fn show_help() -> String {
        Self::show_help_i18n()
    }

    // ✅ SET LEVEL mit i18n - FIXED RETURN TYPE
    pub fn set_level_persistent(level_input: &str) -> std::result::Result<String, String> {
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
                return Err(get_command_translation(
                    "system.commands.log_level.invalid_level",
                    &[level_input],
                ));
            }
        };

        Self::set_level_runtime(level_filter);

        // Async save with i18n error handling - FIXED ERROR HANDLING
        tokio::spawn(async move {
            if let Err(e) = Self::save_to_config(level_filter).await {
                log::warn!(
                    "{}",
                    get_translation("language.service.save_failed", &[&e.to_string()])
                );
            }
        });

        let level_name = Self::level_to_name(level_filter);
        let level_number = Self::level_to_number(level_filter);

        Ok(get_command_translation(
            "system.commands.log_level.changed_success",
            &[&level_name, &level_number],
        ))
    }

    // ✅ Unchanged core functionality
    pub fn set_level_runtime(level_filter: LevelFilter) {
        if let Ok(mut current) = CURRENT_LOG_LEVEL.lock() {
            *current = level_filter;
        }
        log::set_max_level(level_filter);
    }

    pub async fn load_from_config() -> LevelFilter {
        match crate::core::config::Config::load_with_messages(false).await {
            Ok(config) => match Self::string_to_level_filter(&config.log_level) {
                Ok(level) => level,
                Err(_) => {
                    log::warn!(
                        "{}",
                        get_translation(
                            "config.validation.invalid_log_level",
                            &[&config.log_level]
                        )
                    );
                    LevelFilter::Info
                }
            },
            Err(_) => LevelFilter::Info,
        }
    }

    // ✅ FIXED SAVE TO CONFIG - Uses proper AppError
    async fn save_to_config(level_filter: LevelFilter) -> Result<()> {
        match crate::core::config::Config::load_with_messages(false).await {
            Ok(mut config) => {
                config.log_level = Self::level_filter_to_string(level_filter);
                config.save().await
            }
            Err(e) => Err(e),
        }
    }

    // ✅ Unchanged helper methods
    pub fn get_current_level() -> LevelFilter {
        if let Ok(current) = CURRENT_LOG_LEVEL.lock() {
            *current
        } else {
            log::max_level()
        }
    }

    pub fn init_with_level(level: LevelFilter) {
        if let Ok(mut current) = CURRENT_LOG_LEVEL.lock() {
            *current = level;
        }
        log::set_max_level(level);
    }

    // ✅ FIXED STRING TO LEVEL FILTER - Uses proper Result<T>
    fn string_to_level_filter(s: &str) -> std::result::Result<LevelFilter, ()> {
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
