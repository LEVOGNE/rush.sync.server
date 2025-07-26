use crate::core::prelude::*;
use crate::ui::color::AppColor;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, Metadata, Record};
use std::fmt;
use std::sync::{Mutex, PoisonError};

#[derive(Debug)]
pub enum LoggingError {
    LockError(String),
    SetLoggerError(log::SetLoggerError),
}

impl fmt::Display for LoggingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoggingError::LockError(msg) => write!(f, "{}", msg),
            LoggingError::SetLoggerError(err) => write!(f, "{}", err),
        }
    }
}

impl From<log::SetLoggerError> for LoggingError {
    fn from(err: log::SetLoggerError) -> Self {
        LoggingError::SetLoggerError(err)
    }
}

impl<T> From<PoisonError<T>> for LoggingError {
    fn from(_: PoisonError<T>) -> Self {
        LoggingError::LockError(get_translation("system.logging.mutex_error", &[]))
    }
}

lazy_static! {
    static ref LOG_MESSAGES: Mutex<Vec<LogMessage>> = Mutex::new(Vec::new());
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: Level,
    pub message: String,
    pub timestamp: Instant,
}

impl LogMessage {
    pub fn new(level: Level, message: String) -> Self {
        Self {
            level,
            message,
            timestamp: Instant::now(),
        }
    }

    pub fn formatted(&self) -> String {
        let color = AppColor::from_log_level(self.level);
        let level_prefix = format!("[{}]", self.level);

        if self.message.starts_with(&level_prefix) {
            color.format_message("", &self.message)
        } else {
            color.format_message(&self.level.to_string(), &self.message)
        }
    }
}

pub struct AppLogger;

impl AppLogger {
    pub fn get_messages() -> std::result::Result<Vec<LogMessage>, LoggingError> {
        let mut messages = LOG_MESSAGES.lock().map_err(LoggingError::from)?;
        Ok(messages.drain(..).collect())
    }

    fn add_message(level: Level, message: String) -> std::result::Result<(), LoggingError> {
        let mut messages = LOG_MESSAGES.lock().map_err(LoggingError::from)?;
        messages.push(LogMessage::new(level, message));
        Ok(())
    }
}

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let clean_message = record.args().to_string();

            if let Err(e) = Self::add_message(record.level(), clean_message) {
                eprintln!(
                    "{}",
                    get_translation("system.logging.error", &[&e.to_string()])
                );
            }
        }
    }

    fn flush(&self) {}
}

pub fn init() -> std::result::Result<(), log::SetLoggerError> {
    let logger = Box::new(AppLogger);
    log::set_boxed_logger(logger)?;
    log::set_max_level(LevelFilter::Debug);
    Ok(())
}
