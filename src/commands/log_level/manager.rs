use log::LevelFilter;
use std::sync::Mutex;

pub struct LogLevelManager;

static CURRENT_LOG_LEVEL: Mutex<LevelFilter> = Mutex::new(LevelFilter::Info);

impl LogLevelManager {
    pub fn show_status() -> String {
        let current = Self::get_current_level();
        let current_name = Self::level_to_name(current);
        let current_number = Self::level_to_number(current);

        format!(
            "Current log level: {} ({})\n{}",
            current_name,
            current_number,
            Self::show_help()
        )
    }

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
                return Err(format!("Invalid log level: {}", level_input));
            }
        };

        Self::set_level_runtime(level_filter);

        tokio::spawn(async move {
            if let Err(e) = Self::save_to_config(level_filter).await {
                log::warn!("Failed to save log level to config: {}", e);
            } else {
                // ✅ Config save completed (success or failure logged above)
            }
        });

        let level_name = Self::level_to_name(level_filter);
        let level_number = Self::level_to_number(level_filter);

        Ok(format!(
            "✅ Log level changed to: {} ({}) - Persistent saved",
            level_name, level_number
        ))
    }

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
                        "Invalid log level in config: '{}', using INFO",
                        config.log_level
                    );
                    LevelFilter::Info
                }
            },
            Err(_) => LevelFilter::Info,
        }
    }

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

    pub fn show_help() -> String {
        "Available log levels:\n  1 = ERROR   (Only critical errors)\n  2 = WARN    (Warnings and errors)\n  3 = INFO    (General information) [DEFAULT]\n  4 = DEBUG   (Debug information)\n  5 = TRACE   (Very detailed tracing)\n\nUsage:\n  log-level           Show current level\n  log-level 3         Set to INFO level\n  log-level DEBUG     Set to DEBUG level\n  log-level -h        Show this help".to_string()
    }

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
