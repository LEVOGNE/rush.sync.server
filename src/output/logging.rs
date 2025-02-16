// src/logging.rs
use crate::prelude::*;
use std::fmt;
use std::sync::PoisonError;

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
        // Hier KEINE Formatierung mit Level vornehmen
        self.message.clone()
    }
}

pub struct AppLogger;

impl AppLogger {
    pub fn new() -> Self {
        Self
    }

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
            if let Err(e) = Self::add_message(record.level(), record.args().to_string()) {
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
    let logger = Box::new(AppLogger::new());
    log::set_boxed_logger(logger)?;
    log::set_max_level(LevelFilter::Debug);
    Ok(())
}
