// ## BEGIN ##
use crate::core::prelude::*;
use log::{Level, Log, Metadata, Record};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

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
        let mut logs = LOG_MESSAGES.lock().unwrap();
        logs.push(LogMessage::new(Some(level), message));
    }

    pub fn log_plain(message: impl Into<String>) {
        let mut logs = LOG_MESSAGES.lock().unwrap();
        logs.push(LogMessage::new(None, message));
    }

    pub fn get_messages() -> Result<Vec<LogMessage>> {
        let mut logs = LOG_MESSAGES.lock().unwrap();
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
        if self.enabled(record.metadata()) {
            AppLogger::log(record.level(), record.args().to_string());
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<()> {
    if log::set_boxed_logger(Box::new(GlobalLogger)).is_ok() {
        log::set_max_level(log::LevelFilter::Trace);
    }
    Ok(())
}
// ## END ##
