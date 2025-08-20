use crate::core::prelude::*;
use std::io;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Validation(String),
    Terminal(String),
    Translation(TranslationError),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Terminal(err.to_string())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(
                f,
                "{}",
                get_translation("system.error.io_error", &[&err.to_string()])
            ),
            AppError::Validation(msg) => write!(
                f,
                "{}",
                get_translation("system.error.validation_error", &[msg])
            ),
            AppError::Terminal(msg) => write!(
                f,
                "{}",
                get_translation("system.error.terminal_error", &[msg])
            ),
            AppError::Translation(err) => write!(
                f,
                "{}",
                get_translation("system.error.translation_error", &[&err.to_string()])
            ),
        }
    }
}

impl std::error::Error for AppError {}
pub type Result<T> = std::result::Result<T, AppError>;
