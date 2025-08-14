// ## BEGIN ##
use crate::commands::log_level::LogLevelManager;
use crate::core::prelude::*;
use log::{Level, Log, Metadata, Record};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

const MAX_LOG_BUFFER: usize = 1000;
const DRAIN_COUNT: usize = 100;

static LOG_MESSAGES: Lazy<Arc<Mutex<Vec<LogMessage>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: Option<Level>,
    pub message: String,
}

impl LogMessage {
    pub fn new<L: Into<Option<Level>>, S: Into<String>>(level: L, message: S) -> Self {
        Self {
            level: level.into(),
            message: message.into(),
        }
    }

    /// ✅ Statt ANSI → Marker (für dein parse_message_parts)
    pub fn formatted(&self) -> String {
        match self.level {
            Some(level) => format!("[{}] {}", level.to_string().to_uppercase(), self.message),
            None => self.message.clone(),
        }
    }
}

pub struct AppLogger;

impl AppLogger {
    pub fn log(level: Level, message: impl Into<String>) {
        let mut logs = LOG_MESSAGES.lock().unwrap_or_else(|poisoned| {
            log::warn!("Recovered from poisoned mutex");
            poisoned.into_inner()
        });

        // Buffer-Größe begrenzen
        if logs.len() >= MAX_LOG_BUFFER {
            // Entferne die ältesten 100 Einträge
            logs.drain(0..DRAIN_COUNT);

            // Füge Warnung hinzu dass Logs gedroppt wurden
            logs.push(LogMessage::new(
                Some(Level::Warn),
                format!(
                    "[SYSTEM] Dropped {} old log messages to prevent memory overflow",
                    DRAIN_COUNT
                ),
            ));
        }

        logs.push(LogMessage::new(Some(level), message));
    }

    pub fn log_plain(message: impl Into<String>) {
        let mut logs = LOG_MESSAGES.lock().unwrap_or_else(|poisoned| {
            log::warn!("Recovered from poisoned mutex");
            poisoned.into_inner()
        });

        // Buffer-Größe begrenzen
        if logs.len() >= MAX_LOG_BUFFER {
            logs.drain(0..DRAIN_COUNT);
        }

        logs.push(LogMessage::new(None, message));
    }

    pub fn get_messages() -> Result<Vec<LogMessage>> {
        let mut logs = LOG_MESSAGES.lock().unwrap_or_else(|poisoned| {
            log::warn!("Recovered from poisoned mutex");
            poisoned.into_inner()
        });
        let msgs = logs.clone();
        logs.clear();
        Ok(msgs)
    }
}

/// ✅ Globaler Logger hookt alle log::* calls
struct GlobalLogger;

impl Log for GlobalLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        // 🚨 Nachrichten werden nur bei get_messages() geleert
        AppLogger::log(record.level(), record.args().to_string());
    }

    fn flush(&self) {}
}

pub async fn init() -> Result<()> {
    if log::set_boxed_logger(Box::new(GlobalLogger)).is_ok() {
        // ✅ LADE LOG-LEVEL AUS CONFIG
        let config_level = LogLevelManager::load_from_config().await;
        LogLevelManager::init_with_level(config_level);
    }
    Ok(())
}
