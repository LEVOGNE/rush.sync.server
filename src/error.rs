// ## FILE: ./src/error.rs
use std::io;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Validation(String),
    Terminal(String),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Terminal(err.to_string())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "IO-Fehler: {}", err),
            AppError::Validation(msg) => write!(f, "Validierungsfehler: {}", msg),
            AppError::Terminal(msg) => write!(f, "Terminal-Fehler: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

pub type Result<T> = std::result::Result<T, AppError>;
